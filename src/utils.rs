use std::path::{Path, PathBuf};

use anyhow::Result;
use rand::prelude::*;

// List of English adjective words generated based on the data/adjectives.txt file
pub const ADJECTIVES: &[&str] = &include!(concat!(env!("OUT_DIR"), "/adjectives.rs"));

// List of English noun words generated based on the data/nouns.txt file
pub const NOUNS: &[&str] = &include!(concat!(env!("OUT_DIR"), "/nouns.rs"));

pub fn dir_is_writable(path: &Path) -> Result<bool> {
    let meta = path.metadata()?;
    Ok(!meta.permissions().readonly())
}

pub fn gen_name() -> String {
    let mut rng = thread_rng();

    let adj1 = ADJECTIVES.choose(&mut rng).unwrap();
    let adj2 = ADJECTIVES.choose(&mut rng).unwrap();
    let noun = NOUNS.choose(&mut rng).unwrap();

    format!("{}{}{}", adj1, adj2, noun)
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

    url
}

pub fn expand_tilde<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
    let p = path.as_ref();

    if !p.starts_with("~") {
        return Some(p.to_path_buf());
    }
    if p == Path::new("~") {
        return dirs_next::home_dir();
    }
    dirs_next::home_dir().map(|mut h| {
        if h == Path::new("/") {
            // Corner case: `h` root directory;
            // don't prepend extra `/`, just drop the tilde.
            p.strip_prefix("~").unwrap().to_path_buf()
        } else {
            h.push(p.strip_prefix("~/").unwrap());
            h
        }
    })
}
