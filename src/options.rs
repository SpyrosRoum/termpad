use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "termpad")]
pub struct Opt {
    /// Relative or absolute path to the directory where you want to store user-posted pastes.
    #[structopt(short, long, parse(from_os_str), default_value = "./pastes/")]
    pub output: PathBuf,
    /// This will be used as to construct the url that is returned to the client.
    /// Value will be prepended with `http`.
    #[structopt(short, long, default_value = "localhost", env = "DOMAIN_NAME")]
    pub domain: String,
    /// If set, returns the url with `https` prefix instead of `http`.
    #[structopt(long, env = "HTTPS")]
    pub https: bool,
    /// Defines the maximum size (in bytes) of the buffer used for getting data from the user.
    #[structopt(short = "B", long, default_value = "50000", env = "BUFFER_SIZE")]
    pub buffer_size: usize,
}
