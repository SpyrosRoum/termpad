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
