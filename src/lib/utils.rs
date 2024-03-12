use std::path::Path;
use std::time::SystemTime;
use std::{fs, os::unix::fs::MetadataExt};

use anyhow::Result;

use crate::commands::find::{FindResult, FindResults};

pub fn do_find(dir: &Path, name: &str, results: &mut FindResults) -> Result<()> {
    let dir_entries = fs::read_dir(dir)?;

    for e in dir_entries {
        let path = e?.path();
        if path.is_dir() && path.ends_with(name) {
            results.insert(
                path.display().to_string(),
                FindResult {
                    path: path.display().to_string(),
                    size: total_size(&path)?,
                    days_old: days_old(&path)?,
                },
            );
        } else if path.is_dir() {
            do_find(&path, name, results)?;
        }
    }

    Ok(())
}

pub fn days_old(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path)?;
    let now = SystemTime::now();
    let modified = now.duration_since(metadata.modified()?)?;
    let days_ago = modified.as_secs() / (24 * 3600);

    Ok(days_ago)
}

pub fn total_size(path: &Path) -> Result<u64> {
    let mut size: u64 = 0;

    let metadata = fs::metadata(path)?;

    size += metadata.size() as u64;

    if path.is_dir() {
        let entries = fs::read_dir(path)?;

        for e in entries {
            let path = e?.path();

            size += total_size(&path)?;
        }
    }

    Ok(size)
}
