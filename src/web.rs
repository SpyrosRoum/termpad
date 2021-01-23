use std::{
    convert::Infallible,
    io::{BufRead, BufReader},
    sync::Arc,
};

use tokio::runtime::Runtime;
use warp::{self, Filter};

use crate::options::Opt;

pub fn web_main(settings: Arc<Opt>) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let show_paste_route = warp::path!(String)
            .and(warp::get())
            .and(with_settings(settings))
            .and_then(show_paste);

        warp::serve(show_paste_route)
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
    // FixMe Really bad solution, no syntax highlighting, doesn't keep indentation, white
    let mut html = String::from(
        r#"
<html>
    <head>
        <title>termpad</title>
    </head>
    <body>
"#,
    );
    let file_path = settings.output.join(&name.to_lowercase());

    if file_path.is_file() {
        let file = std::fs::File::open(file_path).unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            html.push_str(line.unwrap().as_str());
            html.push_str("</br>");
        }
    } else {
        html.push_str("No luck");
    };
    html.push_str("</body></html>");

    Ok(Box::new(warp::reply::html(html)))
}
