use std::{convert::Infallible, fs, sync::Arc};

use anyhow::Context;
use askama::Template;
use tokio::runtime::Runtime;
use warp::{http::Uri, Filter};

use crate::options::Opt;

const PAGE_404: &str = include_str!("../static/404.html");

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
        let route_404 = warp::path("404").map(|| warp::reply::html(PAGE_404));

        warp::serve(route_404.or(show_paste_route))
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
        Ok(Box::new(warp::redirect(Uri::from_static("/404"))))
    }
}
