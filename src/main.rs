mod options;
mod utils;
mod web;

use std::{
    fs::{self, File},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

use anyhow::{bail, Context, Result};
use log::{error, info, warn};
use simplelog::{Config, LevelFilter, TermLogger, TerminalMode};
use structopt::StructOpt;

use options::Opt;

fn main() -> Result<()> {
    TermLogger::init(LevelFilter::Info, Config::default(), TerminalMode::Mixed)
        .context("Failed to initialise logger.")?;

    let mut opt: Opt = Opt::from_args();
    opt.output = utils::expand_tilde(&opt.output)
        .context("I could not find your home directory, please provide a full path.")
        .unwrap();
    let opt = Arc::new(opt);

    fs::create_dir_all(&opt.output).context(format!(
        "Failed to create directory `{}`",
        opt.output.display()
    ))?;
    match utils::dir_is_writable(&opt.output) {
        Ok(false) | Err(_) => {
            bail!(format!("`{}` is not writeable", opt.output.display()));
        }
        _ => {}
    }
    info!("Using `{}` for saving files", &opt.output.display());

    // This thread listens to port 9999 for accepting input and creating new files.
    let input_thread = {
        let listener = TcpListener::bind(("0.0.0.0", 9999))
            .context("Failed to bind to port 9999 for input listener")?;
        let opt = Arc::clone(&opt);
        thread::Builder::new()
            .name(String::from("Input thread"))
            .spawn(move || {
                for stream in listener.incoming() {
                    if stream.is_err() {
                        warn!(
                            "Could not connect with client for input {:?}",
                            stream.err().unwrap()
                        );
                        continue;
                    }
                    let stream = stream.unwrap();
                    let opt = Arc::clone(&opt);
                    thread::spawn(move || handle_client(stream, opt, true));
                }
            })
            .context("Failed to start Input thread")?
    };
    info!("Listening to port 9999 for input");

    // This thread listens to port 8888 for accepting file names and returning their content.
    let output_thread = {
        let listener = TcpListener::bind(("0.0.0.0", 8888))
            .context("Failed to bind to port 8888 for output listener")?;
        let opt = Arc::clone(&opt);
        thread::Builder::new()
            .name(String::from("Output thread"))
            .spawn(move || {
                for stream in listener.incoming() {
                    if stream.is_err() {
                        warn!(
                            "Could not connect with client for output {:?}",
                            stream.err().unwrap()
                        );
                        continue;
                    }
                    let stream = stream.unwrap();
                    let opt = Arc::clone(&opt);
                    thread::spawn(move || handle_client(stream, opt, false));
                }
            })
            .context("Failed to start Output thread")?
    };
    info!("Listening to port 8888 for output");

    thread::Builder::new()
        .name(String::from("Web thread"))
        .spawn(move || web::web_main(opt))
        .context("Failed to start Web thread")?;

    println!("====================");

    input_thread.join().unwrap();
    output_thread.join().unwrap();
    Ok(())
}

fn handle_client(stream: TcpStream, settings: Arc<Opt>, input: bool) {
    let addr = stream.peer_addr();
    let ip = addr.map_or(String::from("n/a"), |addr| addr.to_string());

    if input {
        info!("Getting input from {}", ip);
        handle_client_input(stream, settings);
    } else {
        info!("Sending output to {}", ip);
        handle_client_output(stream, settings);
    };
}

fn handle_client_input(mut stream: TcpStream, settings: Arc<Opt>) {
    let mut buffer = vec![0; settings.buffer_size];

    let read = stream.read(&mut buffer);
    if read.is_err() {
        error!("Could not read data from input stream.");
        return;
    }
    let read = read.unwrap();

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
    let file = File::create(&file_path);
    if file.is_err() {
        error!(
            "Could not create `{}`: {:?}",
            file_path.display(),
            file.err().unwrap()
        );
        return;
    }
    let mut file = file.unwrap();
    file.write_all(&buffer[..read]).unwrap();

    // The url is usually printed on the other side so we just add a newline here
    let mut url = utils::gen_url(&settings.domain, name.as_str(), settings.https);
    url.push('\n');

    stream
        .write_all(url.as_bytes())
        .map_err(|e| error!("Could not send reply to input request: {:?}", e))
        .ok();
}

fn handle_client_output(mut stream: TcpStream, settings: Arc<Opt>) {
    let mut buffer = [0; 50];

    let read = stream.read(&mut buffer);
    if read.is_err() {
        error!("Could not read data from output stream.");
        return;
    }
    let read = read.unwrap();

    // We do `read - 1` to account for the `\n`
    let file_name = String::from_utf8_lossy(&buffer[..read - 1]).to_lowercase();

    let file_path = settings.output.join(&file_name);
    if file_path.is_file() {
        let content = fs::read_to_string(&file_path);
        if content.is_err() {
            error!(
                "Could not read content from `{}`: {:?}",
                file_path.display(),
                content.err().unwrap()
            );
            return;
        }
        let content = content.unwrap();
        stream
            .write_all(content.as_bytes())
            .map_err(|e| error!("Could not send reply to output request: {:?}", e))
            .ok();
    }
}
