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
