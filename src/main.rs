mod options;
mod utils;

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
        thread::spawn(move || {
            for stream in listener.incoming() {
                if stream.is_err() {
                    continue;
                }
                let stream = stream.unwrap();
                let opt = Arc::clone(&opt);
                thread::spawn(move || handle_client(stream, opt, true));
            }
        })
    };

    // This thread listens to port 8888 for accepting file names and returning their content.
    let output_thread = {
        let listener = TcpListener::bind(("0.0.0.0", 8888))?;
        let opt = Arc::clone(&opt);
        thread::spawn(move || {
            for stream in listener.incoming() {
                if stream.is_err() {
                    continue;
                }
                let stream = stream.unwrap();
                let opt = Arc::clone(&opt);
                thread::spawn(move || handle_client(stream, opt, false));
            }
        })
    };

    input_thread.join().unwrap();
    output_thread.join().unwrap();
    Ok(())
}

fn handle_client(stream: TcpStream, settings: Arc<Opt>, input: bool) {
    if input {
        handle_client_input(stream, settings);
    } else {
        handle_client_output(stream, settings);
    }
}

fn handle_client_input(mut stream: TcpStream, settings: Arc<Opt>) {
    let mut buffer = vec![0; settings.buffer_size];

    let read = stream.read(&mut buffer).unwrap();

    let mut file = settings.output.clone();
    let mut name = loop {
        let name = utils::gen_name();
        file.push(name.to_lowercase());
        if !file.is_file() {
            break name;
        }
        file.pop();
    };
    let mut file = File::create(file).unwrap();
    file.write_all(&buffer[..read]).unwrap();

    name.push('\n');
    stream.write_all(name.as_bytes()).unwrap();
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
