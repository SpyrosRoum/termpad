use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "termpad")]
pub struct Opt {
    /// Relative or absolute path to the directory where you want to store user-posted pastes.
    #[structopt(short, long, parse(from_os_str), default_value = "./pastes/")]
    pub output: PathBuf,
    /// This will be used as a prefix for an output received by the client.
    /// Value will be prepended with `http`.
    #[structopt(short, long, default_value = "localhost")]
    pub domain: String,
    /// If set, returns the url with `https` prefix instead of `http`.
    #[structopt(short, long)]
    pub https: bool,
    /// This parameter defines size of the buffer used for getting data from the user.
    /// Maximum size (in bytes) of all input files is defined by this value.
    #[structopt(short = "B", long, default_value = "50000")]
    pub buffer_size: usize,
}
