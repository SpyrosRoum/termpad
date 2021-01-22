use std::path::Path;

use anyhow::Result;
use rand::prelude::*;

/// List of English adjective words
pub const ADJECTIVES: &[&str] = &include!(concat!(env!("OUT_DIR"), "/adjectives.rs"));

/// List of English noun words
pub const NOUNS: &[&str] = &include!(concat!(env!("OUT_DIR"), "/nouns.rs"));

pub fn dir_is_writable(path: &Path) -> Result<bool> {
    let meta = path.metadata()?;
    Ok(!meta.permissions().readonly())
}

pub fn gen_name() -> String {
    let mut rng = thread_rng();

    let adj = ADJECTIVES.choose(&mut rng).unwrap();
    let noun = NOUNS.choose(&mut rng).unwrap();

    format!("{}{}", adj, noun)
}

pub fn gen_url(domain: &str, name: &str, https: bool) -> String {
    let mut url = if https {
        String::from("https://")
    } else {
        String::from("http://")
    };

    url.push_str(domain);
    if !domain.ends_with('/') {
        url.push('/');
    }
    url.push_str(name);
    url.push('\n');

    url
}
