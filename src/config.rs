use std::{env, path::PathBuf, str::FromStr};

use {
    anyhow::{bail, Context},
    once_cell::sync::Lazy,
};

use crate::utils;

pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::read_config().unwrap());

#[derive(Debug)]
pub struct Config {
    /// Relative or absolute path to the directory where you want to store user-posted pastes.
    pub output: PathBuf,
    /// This will be used as to construct the url that is returned to the client.
    /// Value will be prepended with `http`.
    pub domain: String,
    /// If set, returns the url with `https` prefix instead of `http`.
    pub https: bool,
    /// The port on which the app should run on
    pub port: u16,
    /// How many days to keep files for. A value of 0 means forever
    pub delete_after: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output: "default".into(),
            domain: "localhost".to_string(),
            https: false,
            port: 8000,
            delete_after: 120,
        }
    }
}

impl Config {
    /// Read config from the environment.
    /// Only the `output` needs to be set, the rest are optional
    pub fn read_config() -> anyhow::Result<Self> {
        let mut conf = Config::default();

        if let Ok(o) = env::var("OUTPUT") {
            let out = utils::expand_tilde(&o);
            if out.is_none() {
                bail!("Failed to expand the tilde `~`, please explicitly write your home dir.");
            }
            let out = out.unwrap();
            conf.output = out;
        } else {
            bail!("`OUTPUT` needs to be set");
        }

        if let Ok(v) = env::var("DOMAIN_NAME") {
            conf.domain = v;
        }
        if let Ok(v) = env::var("HTTPS") {
            conf.https = bool::from_str(&v).context(format!("`{}` is not a bool", v))?;
        }
        if let Ok(v) = env::var("PORT") {
            conf.port = u16::from_str(&v).context(format!("`{}` is not a valid port", v))?;
        }
        if let Ok(v) = env::var("DELETE_AFTER") {
            conf.delete_after =
                u32::from_str(&v).context(format!("`{}` is not a valid number", v))?;
        }

        if conf.domain == "localhost" {
            conf.domain = format!("localhost:{}", conf.port);
        }

        Ok(conf)
    }
}
