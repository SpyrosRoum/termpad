mod config;
mod error;
mod handlers;
mod templates;
mod utils;

use std::{fs, net::SocketAddr};

use anyhow::{bail, Context};
use axum::{routing::get, Router};
use simplelog::{LevelFilter, TermLogger, TerminalMode};

use config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    TermLogger::init(
        LevelFilter::Info,
        simplelog::Config::default(),
        TerminalMode::Mixed,
    )
    .context("Failed to initialise logger.")?;

    let config = Config::read_config()?;

    fs::create_dir_all(&config.output).context(format!(
        "Failed to create directory `{}`",
        config.output.display()
    ))?;
    if !utils::dir_is_writable(&config.output) {
        log::error!("{} is not writeable, exiting", &config.output.display());
        bail!("{} is not writeable, exiting", config.output.display());
    }
    log::info!("Using `{}` for saving files", &config.output.display());

    if config.delete_after != 0 {
        utils::clean_files_task(&config.output, config.delete_after);
    }

    let app = Router::new()
        .route("/raw/:key", get(handlers::get_raw))
        .route("/:key", get(handlers::get_web))
        .route("/usage", get(handlers::usage))
        .route("/", get(handlers::web_paste).post(handlers::upload));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    log::info!("Listening on: 0.0.0.0:{}", config.port);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("Failed to start axum::Server")
}

// #[post("/", data = "<paste>")]
// fn upload(paste: Data, settings: State<Config>) -> Result<String, Debug<io::Error>> {
//     let mut file_path = settings.output.clone();
//     let name = loop {
//         let name = utils::gen_name();
//         file_path.push(name.to_lowercase());
//         if !file_path.is_file() {
//             break name;
//         }
//         // Since the file exists we remove the last part so we can get a new one on the next iteration
//         file_path.pop();
//     };
//     file_path.set_extension("zst");

//     let file = File::create(file_path)?;
//     // Level 0 uses uses zstd's default (At the moment of writing, 3)
//     zstd::stream::copy_encode(paste.open(), file, 0)?;

//     let mut url = utils::gen_url(&settings.domain, name.as_str(), settings.https);
//     url.push('\n');

//     Ok(url)
// }

// #[get("/<key>")]
// fn retrieve(key: String, settings: State<Config>) -> Result<content::Html<String>, Redirect> {
//     let file_path = {
//         let mut path = settings.output.join(key.to_lowercase());
//         path.set_extension("zst");
//         path
//     };

//     if !file_path.is_file() {
//         return Err(Redirect::to("/usage?not_found=true"));
//     }

//     let code = {
//         let mut dest = Vec::new();
//         zstd::stream::copy_decode(File::open(file_path).unwrap(), &mut dest)
//             .map_err(|_| Redirect::to("/usage?not_found=true"))?;
//         String::from_utf8_lossy(&dest).to_string()
//     };

//     let template = templates::PasteTemplate { code };
//     match template.render() {
//         Ok(html) => Ok(content::Html(html)),
//         _ => Err(Redirect::to("/usage?not_found=true")),
//     }
// }
