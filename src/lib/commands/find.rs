use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use bytesize::ByteSize;

use super::{Commands, TreeWalk};
use crate::utils::do_find;

pub struct FindResult {
    pub path: String,
    pub size: u64,
    pub days_old: u64,
}

pub type FindResults = HashMap<String, FindResult>;

pub fn find_command(command: &TreeWalk) -> Result<()> {
    if let Commands::Find(ref args) = &command.command {
        let mut results = FindResults::new();

        let from_path = Path::new(&command.from);

        do_find(from_path, &args.name, &mut results)?;

        let sorted_results = sorted_results(&results, command.sort_by_age);
        let path_width = longest_path(&results);

        // Filter results

        let mut found_dirs: u64 = 0;
        let mut found_bytes: u64 = 0;
        let mut skipped_dirs: u64 = 0;

        println!("");

        for result in &sorted_results {
            if result.days_old >= command.min_age_days
                && result.size >= ByteSize::mb(command.min_mb).0
            {
                found_dirs += 1;
                found_bytes += result.size;

                println!(
                    "{:<path_width$} | uses {:10} | {:4} days old",
                    result.path,
                    ByteSize::b(result.size).to_string_as(true),
                    result.days_old,
                );
            } else {
                skipped_dirs += 1;
            }
        }

        let total_size = sorted_results.iter().fold(0, |t, e| t + e.size);

        println!(
            "\nFound   {} directories {}, total size {}",
            found_dirs,
            &args.name,
            ByteSize::b(found_bytes).to_string_as(true)
        );

        println!(
            "Skipped {} directories {}, total size {}",
            skipped_dirs,
            &args.name,
            ByteSize::b(total_size - found_bytes).to_string_as(true)
        );
    } else {
        anyhow::bail!("Incorrect args for 'find'");
    }

    Ok(())
}

fn sorted_results(results: &FindResults, sort_by_age: bool) -> Vec<&FindResult> {
    let mut results: Vec<_> = results.values().collect();

    if sort_by_age {
        results.sort_by(|a, b| b.days_old.cmp(&a.days_old));
    } else {
        results.sort_by(|a, b| b.size.cmp(&a.size));
    }

    results
}

fn longest_path(results: &FindResults) -> usize {
    results
        .iter()
        .fold(0, |acc, result| usize::max(acc, result.1.path.len()))
}
