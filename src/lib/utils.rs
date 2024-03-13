use std::path::Path;
use std::time::SystemTime;
use std::{fs, os::unix::fs::MetadataExt};

use anyhow::{Context, Result};

use crate::commands::find::{FindResult, FindResults};

pub fn do_find(dir: &Path, name: &str, results: &mut FindResults) -> Result<()> {
    let dir_entries =
        fs::read_dir(dir).context(format!("Failed to open dir: {}", dir.display()))?;

    for e in dir_entries {
        let path = e?.path();

        if !path.is_symlink() {
            if path.is_dir() && path.ends_with(name) {
                results.insert(
                    path.display().to_string(),
                    FindResult {
                        path: path.display().to_string(),
                        size: total_size(&path)?,
                        days_old: earliest_modified(path.parent().unwrap())?,
                    },
                );
            } else if path.is_dir() {
                do_find(&path, name, results)?;
            }
        }
    }

    Ok(())
}

pub fn days_old(path: &Path) -> Result<u64> {
    if !path.is_symlink() {
        let metadata = fs::metadata(path).context(format!(
            "Failed to get metadata for path: {}",
            path.display()
        ))?;

        let now = SystemTime::now();
        let modified = now.duration_since(metadata.modified()?)?;
        let days_ago = modified.as_secs() / (24 * 3600);

        Ok(days_ago)
    } else {
        Ok(u64::MAX)
    }
}

pub fn earliest_modified(path: &Path) -> Result<u64> {
    let mut earliest_path = "".to_string();

    let age = do_earliest_modified(path, u64::MAX, &mut earliest_path)?;

    Ok(age)
}

fn do_earliest_modified(path: &Path, result: u64, result_path: &mut String) -> Result<u64> {
    let mut new_result = result;

    let age = days_old(path)?;

    if age < new_result {
        new_result = age;
        result_path.replace_range(.., path.to_str().unwrap_or("BAD-PATH"));
    }

    if new_result == 0 {
        return Ok(new_result);
    } else {
        if path.is_dir() {
            let entries = fs::read_dir(path)?;

            for e in entries {
                let path = e?.path();

                let age = do_earliest_modified(&path, result, result_path)?;

                if age < new_result {
                    new_result = age;
                }

                if new_result == 0 {
                    return Ok(new_result);
                }
            }
        }
    }

    Ok(new_result)
}

pub fn total_size(path: &Path) -> Result<u64> {
    if !path.is_symlink() {
        if path.is_file() {
            let metadata = fs::metadata(path).context(format!(
                "Failed to get metadata for path: {}",
                path.display()
            ))?;

            return Ok(metadata.size() as u64);
        } else if path.is_dir() {
            let mut size: u64 = 0;

            let entries = fs::read_dir(path)?;

            for e in entries {
                let path = e?.path();

                size += total_size(&path)?;
            }

            return Ok(size);
        }
    }

    return Ok(0);
}
