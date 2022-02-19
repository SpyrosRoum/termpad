use {
    askama::Template,
    async_compression::tokio::bufread::ZstdDecoder,
    axum::{
        body::StreamBody,
        extract::{Path, Query},
        response::{Html, IntoResponse},
    },
    serde::Deserialize,
    tokio::{
        fs::File,
        io::{AsyncReadExt, BufReader},
    },
    tokio_util::io::ReaderStream,
};

use crate::{config::CONFIG, error::Error, templates};

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

pub async fn upload() -> impl IntoResponse {
    todo!()
}
