#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

mod options;
mod templates;
mod utils;

use std::{
    fs::{self, File},
    io,
};

use anyhow::{bail, Context};
use askama::Template;
use log::{error, info};
use rocket::{
    config::{ConfigBuilder, Environment},
    response::{content, Debug, Redirect, Stream},
    Data, State,
};
use simplelog::{Config, LevelFilter, TermLogger, TerminalMode};
use structopt::StructOpt;

use options::Opt;

const INPUT_PAGE: &str = include_str!("../static/input.html");

fn main() -> anyhow::Result<()> {
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed)
        .context("Failed to initialise logger.")?;

    let mut opt: Opt = Opt::from_args();
    opt.output = utils::expand_tilde(&opt.output)
        .context("I could not find your home directory, please provide a full path.")
        .unwrap();
    if opt.domain == "localhost" {
        opt.domain = format!("localhost:{}", opt.port);
    }
    // So it's not mutable anymore
    let opt = opt;

    fs::create_dir_all(&opt.output).context(format!(
        "Failed to create directory `{}`",
        opt.output.display()
    ))?;
    if !utils::dir_is_writable(&opt.output) {
        error!("{} is not writeable, exiting", &opt.output.display());
        bail!("{} is not writeable, exiting", opt.output.display());
    }
    info!("Using `{}` for saving files", &opt.output.display());

    if opt.delete_after != 0 {
        utils::clean_files_task(&opt.output, opt.delete_after);
    }

    let config = ConfigBuilder::new(Environment::Production)
        .address("0.0.0.0")
        .port(opt.port)
        .finalize()
        .context("Failed to configure Rocket")?;
    rocket::custom(config)
        .mount(
            "/",
            routes![usage, upload, web_input, retrieve_raw, retrieve],
        )
        .manage(opt)
        .launch();

    Ok(())
}

#[get("/")]
fn web_input() -> content::Html<String> {
    content::Html(INPUT_PAGE.to_string())
}

#[post("/", data = "<paste>")]
fn upload(paste: Data, settings: State<Opt>) -> Result<String, Debug<io::Error>> {
    let mut file_path = settings.output.clone();
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

    let file = File::create(file_path)?;
    // Level 0 uses uses zstd's default (At the moment of writing, 3)
    zstd::stream::copy_encode(paste.open(), file, 0)?;

    let mut url = utils::gen_url(&settings.domain, name.as_str(), settings.https);
    url.push('\n');

    Ok(url)
}

#[get("/<key>")]
fn retrieve(key: String, settings: State<Opt>) -> Result<content::Html<String>, Redirect> {
    let file_path = {
        let mut path = settings.output.join(key.to_lowercase());
        path.set_extension("zst");
        path
    };

    if !file_path.is_file() {
        return Err(Redirect::to("/usage?not_found=true"));
    }

    let code = {
        let mut dest = Vec::new();
        zstd::stream::copy_decode(File::open(file_path).unwrap(), &mut dest)
            .map_err(|_| Redirect::to("/usage?not_found=true"))?;
        String::from_utf8_lossy(&dest).to_string()
    };

    let template = templates::PasteTemplate { code };
    match template.render() {
        Ok(html) => Ok(content::Html(html)),
        _ => Err(Redirect::to("/usage?not_found=true")),
    }
}

#[get("/raw/<key>")]
fn retrieve_raw(key: String, settings: State<Opt>) -> Result<Stream<impl io::Read>, &'static str> {
    let file_path = {
        let mut path = settings.output.join(key.to_ascii_lowercase());
        path.set_extension("zst");
        path
    };
    let decoder = {
        let file = File::open(file_path).map_err(|_| "")?;
        zstd::Decoder::new(file).map_err(|_| "")?
    };

    Ok(Stream::from(decoder))
}

#[get("/usage?<not_found>")]
fn usage(not_found: Option<bool>, settings: State<Opt>) -> content::Html<String> {
    let template = templates::IndexTemplate {
        not_found: not_found.unwrap_or(false),
        domain: settings.domain.clone(),
        delete_after: settings.delete_after,
    };
    match template.render() {
        Ok(html) => content::Html(html),
        _ => content::Html(
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
        ),
    }
}
