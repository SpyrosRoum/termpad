use std::{convert::Infallible, fs, sync::Arc};

use anyhow::Context;
use askama::Template;
use tokio::runtime::Runtime;
use warp::{http::Uri, Filter};

use crate::options::Opt;

#[derive(Template)]
#[template(path = "index.html")]
struct PasteTemplate {
    code: String,
}

pub fn web_main(settings: Arc<Opt>) {
    let rt = Runtime::new()
        .context("Failed to start tokio runtime")
        .unwrap();

    rt.block_on(async {
        let show_paste_route = warp::path!(String)
            .and(warp::get())
            .and(with_settings(settings))
            .and_then(show_paste);
        let static_files_route = warp::path("static").and(warp::fs::dir("static/"));
        let route_404 = warp::path("404").and_then(no_file);

        warp::serve(route_404.or(show_paste_route).or(static_files_route))
            .run(([0, 0, 0, 0], 3030))
            .await;
    });
}

fn with_settings(
    settings: Arc<Opt>,
) -> impl Filter<Extract = (Arc<Opt>,), Error = Infallible> + Clone {
    warp::any().map(move || settings.clone())
}

async fn show_paste(name: String, settings: Arc<Opt>) -> Result<Box<dyn warp::Reply>, Infallible> {
    let file_path = settings.output.join(&name.to_lowercase());
    if file_path.is_file() {
        let code = fs::read_to_string(file_path)
            .context("Failed to read paste")
            .unwrap();
        let template = PasteTemplate { code };
        let html = template
            .render()
            .context("Failed to render html template")
            .unwrap();
        Ok(Box::new(warp::reply::html(html)))
    } else {
        Ok(Box::new(warp::redirect(Uri::from_static(
            "/static/404.html",
        ))))
    }
}

// 404 handler
async fn no_file() -> Result<Box<dyn warp::Reply>, Infallible> {
    Ok(Box::new(warp::reply::html("html")))
}
