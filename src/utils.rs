#![allow(clippy::collapsible_if)]

use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    thread,
    time::{Duration, SystemTime},
};

use log::{error, info, warn};
use rand::prelude::*;

// List of English adjective words generated based on the data/adjectives.txt file
pub(crate) const ADJECTIVES: &[&str] = &include!(concat!(env!("OUT_DIR"), "/adjectives.rs"));

// List of English noun words generated based on the data/nouns.txt file
pub(crate) const NOUNS: &[&str] = &include!(concat!(env!("OUT_DIR"), "/nouns.rs"));

const SECS_IN_DAY: u64 = 86400;

pub(crate) fn dir_is_writable(path: &Path) -> bool {
    let file_path = path.join("test_permissions_file");
    File::create(file_path).is_ok()
}

pub(crate) fn gen_name() -> String {
    let mut rng = thread_rng();

    let adj1 = ADJECTIVES.choose(&mut rng).unwrap();
    let adj2 = ADJECTIVES.choose(&mut rng).unwrap();
    let noun = NOUNS.choose(&mut rng).unwrap();

    format!("{}{}{}", adj1, adj2, noun)
}

pub(crate) fn gen_url(domain: &str, name: &str, https: bool) -> String {
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

pub(crate) fn expand_tilde<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
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

pub(crate) fn clean_files_task(out: &Path, delete_after: u32) {
    let out = out.to_owned();
    let dur = Duration::from_secs(2 * SECS_IN_DAY);
    thread::spawn(move || loop {
        info!("Cleaning files..");
        info!("Deleted {} files", clean_files(&out, delete_after));
        thread::sleep(dur);
    });
}

/// `delete_after` is in days
fn clean_files<P: AsRef<Path>>(out: P, delete_after: u32) -> u32 {
    let secs = delete_after as u64 * SECS_IN_DAY;
    let keep_for = Duration::from_secs(secs);
    let now = SystemTime::now();

    let mut counter = 0;
    for file in fs::read_dir(out).unwrap().map(|f| f.unwrap()) {
        let meta = file.metadata();
        if meta.is_err() {
            warn!(
                "I don't have permissions to read metadata for `{}`.",
                file.path().display()
            );
            continue;
        }
        let meta = meta.unwrap();
        if !meta.is_file() {
            // We only create files so this is certainly not ours
            continue;
        }

        let created = meta.created();
        if created.is_err() {
            error!("Your platform or filesystem doesn't support reading creation date of files so I can't delete them.");
            break;
        }
        let created = created.unwrap();

        if created + keep_for <= now {
            if fs::remove_file(file.path()).is_err() {
                warn!("Failed to delete old file `{}`", file.path().display())
            } else {
                counter += 1;
            };
        }
    }
    counter
}
