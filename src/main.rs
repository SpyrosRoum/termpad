mod options;
mod utils;
#[cfg(feature = "web")]
mod web;

use std::{
    fs::{create_dir_all, File},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

use anyhow::{bail, Context, Result};
use structopt::StructOpt;

use options::Opt;

fn main() -> Result<()> {
    let opt: Arc<Opt> = Arc::new(Opt::from_args());

    create_dir_all(&opt.output).context(format!(
        "Failed to create directory `{}`",
        opt.output.display()
    ))?;
    match utils::dir_is_writable(&opt.output) {
        Ok(false) | Err(_) => {
            bail!(format!("`{}` is not writeable", opt.output.display()));
        }
        _ => {}
    }

    // This thread listens to port 9999 for accepting input and creating new files.
    let input_thread = {
        let listener = TcpListener::bind(("0.0.0.0", 9999))?;
        let opt = Arc::clone(&opt);
        thread::Builder::new().name(String::from("Input thread")).spawn(move || {
            for stream in listener.incoming() {
                if stream.is_err() {
                    continue;
                }
                let stream = stream.unwrap();
                let opt = Arc::clone(&opt);
                thread::spawn(move || handle_client(stream, opt, true));
            }
        }).context("Failed to start Input thread")?
    };
    println!("Listening to port 9999 for input");

    // This thread listens to port 8888 for accepting file names and returning their content.
    let output_thread = {
        let listener = TcpListener::bind(("0.0.0.0", 8888))?;
        let opt = Arc::clone(&opt);
        thread::Builder::new().name(String::from("Output thread")).spawn(move || {
            for stream in listener.incoming() {
                if stream.is_err() {
                    continue;
                }
                let stream = stream.unwrap();
                let opt = Arc::clone(&opt);
                thread::spawn(move || handle_client(stream, opt, false));
            }
        }).context("Failed to start Output thread")?
    };
    println!("Listening to port 8888 for output");

    #[cfg(feature = "web")]
    {
        let opt = Arc::clone(&opt);
        thread::Builder::new().name(String::from("Web thread")).spawn(move || web::web_main(opt)).context("Failed to start Web thread")?;
        println!("Web server running in port 3030");
    }
    println!("====================");

    input_thread.join().unwrap();
    output_thread.join().unwrap();
    Ok(())
}

fn handle_client(stream: TcpStream, settings: Arc<Opt>, input: bool) {
    let addr = stream.peer_addr();
    let ip = addr.map_or(String::from("n/a"), |addr| addr.to_string());

    let direction = if input {
        handle_client_input(stream, settings);
        "Getting input from"
    } else {
        handle_client_output(stream, settings);
        "Sending output to"
    };

    println!("{} {}", direction, ip);
}

fn handle_client_input(mut stream: TcpStream, settings: Arc<Opt>) {
    let mut buffer = vec![0; settings.buffer_size];

    let read = stream.read(&mut buffer).unwrap();

    let mut file = settings.output.clone();
    let name = loop {
        let name = utils::gen_name();
        file.push(name.to_lowercase());
        if !file.is_file() {
            break name;
        }
        // Since the file exists we remove the last part so we can get a new one on the next iteration
        file.pop();
    };
    let mut file = File::create(file).unwrap();
    file.write_all(&buffer[..read]).unwrap();

    // The answer is usually printed on the other side so we just add a newline here
    let mut answer = String::new();
    #[cfg(feature = "web")]
    {
        let url = utils::gen_base_url(&settings.domain, settings.https);
        answer.push_str(url.as_str());
    }
    answer.push_str(name.as_str());
    answer.push('\n');
    stream.write_all(answer.as_bytes()).unwrap();
}

fn handle_client_output(mut stream: TcpStream, settings: Arc<Opt>) {
    let mut buffer = [0; 50];

    let read = stream.read(&mut buffer).unwrap();

    // We do `read - 1` to account for the `\n`
    let file_name = String::from_utf8_lossy(&buffer[..read - 1]).to_lowercase();

    let file_path = settings.output.join(&file_name);
    if file_path.is_file() {
        let mut file = File::open(file_path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        stream.write_all(content.as_bytes()).unwrap();
    }
}
