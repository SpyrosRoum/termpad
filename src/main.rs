#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

mod options;
mod utils;

use std::{fs, io};

use anyhow::{bail, Context};
use log::{error, info};
use rocket::{
    http::Status,
    response::{content, Debug},
    Data, Request, State,
};
use simplelog::{Config, LevelFilter, TermLogger, TerminalMode};
use structopt::StructOpt;

use options::Opt;

const PAGE_404: &str = include_str!("../static/404.html");

fn main() -> anyhow::Result<()> {
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed)
        .context("Failed to initialise logger.")?;

    let mut opt: Opt = Opt::from_args();
    opt.output = utils::expand_tilde(&opt.output)
        .context("I could not find your home directory, please provide a full path.")
        .unwrap();
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

    rocket::ignite()
        .mount("/", routes![upload, retrieve, retrieve_raw])
        .manage(opt)
        .register(catchers![not_found])
        .launch();

    Ok(())
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
    paste.stream_to_file(file_path)?;
    let mut url = utils::gen_url(&settings.domain, name.as_str(), settings.https);
    url.push('\n');
    Ok(url)
}

#[get("/<key>")]
fn retrieve(key: String, settings: State<Opt>) -> Result<content::Html<String>, Status> {
    utils::actual_retrieve(&settings.output, &key, false)
        .map_or_else(|| Err(Status::NotFound), |html| Ok(content::Html(html)))
}

#[get("/raw/<key>")]
fn retrieve_raw(key: String, settings: State<Opt>) -> String {
    utils::actual_retrieve(&settings.output, &key, true).unwrap_or_else(|| String::from(""))
}

#[catch(404)]
fn not_found(_req: &Request) -> content::Html<&'static str> {
    content::Html(PAGE_404)
}
