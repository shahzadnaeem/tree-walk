use std::path::Path;
use std::time::SystemTime;
use std::{fs, os::unix::fs::MetadataExt};

use anyhow::{Context, Result};
use bytesize::ByteSize;

use crate::commands::{FindArgs, TreeWalk};

pub struct FindResult {
    pub path: String,
    pub size: u64,
    pub days_old: u64,
}

pub type FindResults = Vec<FindResult>;

pub struct ProcessedResults {
    pub results: FindResults,
    pub longest_path: usize,
    pub found_dirs: u64,
    pub found_bytes: u64,
    pub skipped_dirs: u64,
    pub skipped_bytes: u64,
}

pub fn find(command: &TreeWalk, args: &FindArgs) -> Result<ProcessedResults> {
    let from_path = Path::new(&command.from);

    let mut results = Vec::new();

    do_find(from_path, args, &mut results)?;

    let results = process_results(
        results,
        command.min_mb,
        command.min_age_days,
        command.sort_by_age,
    );

    Ok(results)
}

fn do_find(from_path: &Path, args: &FindArgs, results: &mut FindResults) -> Result<()> {
    let dir_entries =
        fs::read_dir(from_path).context(format!("Failed to open dir: {}", from_path.display()))?;

    for e in dir_entries {
        let path = e?.path();

        if !path.is_symlink() {
            if path.is_dir() && path.ends_with(&args.name) {
                results.push(FindResult {
                    path: path.display().to_string(),
                    size: total_size(&path)?,
                    days_old: earliest_modified(path.parent().unwrap())?,
                });
            } else if path.is_dir() {
                do_find(&path, args, results)?;
            }
        }
    }

    Ok(())
}

fn process_results(
    mut results: FindResults,
    min_mb: u64,
    min_age_days: u64,
    sort_by_age: bool,
) -> ProcessedResults {
    sort_results(&mut results, sort_by_age);

    // Filter results

    let orig_num_dirs = results.len();
    let orig_total_bytes = results.iter().fold(0, |acc, e| acc + e.size);

    let filtered_results: FindResults = results
        .into_iter()
        .filter(|result| result.days_old >= min_age_days && result.size >= ByteSize::mb(min_mb).0)
        .collect();

    let found_dirs = filtered_results.len();
    let found_bytes = filtered_results.iter().fold(0, |acc, e| acc + e.size);
    let skipped_dirs = (orig_num_dirs - found_dirs) as u64;

    ProcessedResults {
        longest_path: longest_path(&filtered_results),
        found_dirs: found_dirs as u64,
        found_bytes,
        skipped_dirs,
        skipped_bytes: orig_total_bytes - found_bytes,
        results: filtered_results,
    }
}

fn sort_results(results: &mut FindResults, sort_by_age: bool) {
    if sort_by_age {
        results.sort_by(|a, b| b.days_old.cmp(&a.days_old));
    } else {
        results.sort_by(|a, b| b.size.cmp(&a.size));
    }
}

fn longest_path(results: &FindResults) -> usize {
    results
        .iter()
        .fold(0, |acc, result| usize::max(acc, result.path.len()))
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
