use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "termpad")]
pub struct Opt {
    /// Relative or absolute path to the directory where you want to store user-posted pastes.
    #[structopt(
        short,
        long,
        parse(from_os_str),
        default_value = "~/.local/share/termpad",
        env = "OUTPUT"
    )]
    pub output: PathBuf,
    /// This will be used as to construct the url that is returned to the client.
    /// Value will be prepended with `http`.
    #[structopt(short, long, default_value = "localhost", env = "DOMAIN_NAME")]
    pub domain: String,
    /// If set, returns the url with `https` prefix instead of `http`.
    #[structopt(long, env = "HTTPS")]
    pub https: bool,
    /// The port on which the app should run on
    #[structopt(short, long, default_value = "8000", env="PORT")]
    pub port: u16,
    /// How many days to keep files for. A value of 0 means forever
    #[structopt(long, default_value = "120", env = "DELETE_AFTER")]
    pub delete_after: u32,
}
