use std::io;

use {
    askama::Template,
    async_compression::tokio::bufread::{ZstdDecoder, ZstdEncoder},
    axum::{
        body::StreamBody,
        extract::{BodyStream, Path, Query},
        response::{Html, IntoResponse},
    },
    futures::stream::StreamExt,
    serde::Deserialize,
    tokio::{
        fs::File,
        io::{AsyncReadExt, AsyncWriteExt, BufReader},
    },
    tokio_util::io::{ReaderStream, StreamReader},
};

use crate::{config::CONFIG, error::Error, templates, utils};

const INPUT_PAGE: &str = include_str!("../static/input.html");

pub async fn get_raw(Path(key): Path<String>) -> Result<impl IntoResponse, Error> {
    let file_path = {
        let mut path = CONFIG.output.join(key.to_ascii_lowercase());
        path.set_extension("zst");
        path
    };
    let decoder = {
        let file = File::open(file_path).await?;
        let buffered = BufReader::new(file);
        ZstdDecoder::new(buffered)
    };

    let stream = ReaderStream::new(decoder);
    Ok(StreamBody::new(stream))
}

pub async fn get_web(Path(key): Path<String>) -> Result<impl IntoResponse, Error> {
    let file_path = {
        let mut path = CONFIG.output.join(key.to_ascii_lowercase());
        path.set_extension("zst");
        path
    };

    if !file_path.is_file() {
        return Err(Error::NotFound);
    }

    let mut decoder = {
        let file = File::open(file_path).await?;
        let buffered = BufReader::new(file);
        ZstdDecoder::new(buffered)
    };

    let mut buff = Vec::new();
    decoder.read_to_end(&mut buff).await?;
    let code = String::from_utf8_lossy(&buff).to_string();

    let template = templates::PasteTemplate { code };
    match template.render() {
        Ok(html) => Ok(Html(html)),
        _ => Err(Error::NotFound),
    }
}

#[derive(Deserialize)]
pub struct UsageQuery {
    not_found: bool,
}

pub async fn usage(not_found: Option<Query<UsageQuery>>) -> Result<Html<String>, Error> {
    let not_found = not_found.map(|v| v.not_found).unwrap_or(false);

    let template = templates::IndexTemplate {
        not_found,
        domain: CONFIG.domain.clone(),
        delete_after: CONFIG.delete_after,
    };
    Ok(template.render().map(Html).unwrap_or_else(|_| {
        Html(
            r#"
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <title>termpad</title>
        </head>
        <body style="background-color:#282a36">
        <h2 style="color:#ccc"> Something went wrong </h2>
        </body>"#
                .to_string(),
        )
    }))
}

pub async fn web_paste() -> Html<&'static str> {
    Html(INPUT_PAGE)
}

pub async fn upload(paste: BodyStream) -> Result<String, Error> {
    let mut file_path = CONFIG.output.clone();
    let name = loop {
        let name = utils::gen_name();
        file_path.push(name.to_lowercase());
        if !file_path.is_file() {
            break name;
        }
        // Since the file exists we remove the last part so we can get a new one on the next iteration
        file_path.pop();
    };
    file_path.set_extension("zst");

    let mut file = File::create(file_path).await?;

    let reader = StreamReader::new(
        paste.map(|res| res.map_err(|e| io::Error::new(io::ErrorKind::Other, e))),
    );
    let mut enc = ZstdEncoder::new(reader);

    let mut buff = [0; 255];
    loop {
        let read = enc.read(&mut buff).await?;
        if read == 0 {
            break;
        }

        let mut wrote_total = 0;
        while wrote_total < read {
            let wrote = file.write(&buff[0..read]).await?;
            if wrote == 0 {
                // For some reason we can't write to the file
                return Err(Error::Other("Failed to write to file"));
            }
            wrote_total += wrote;
        }
    }

    let url = utils::gen_url(&CONFIG.domain, &name, CONFIG.https).map(|u| u.to_string())?;
    Ok(url)
}
