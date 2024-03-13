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
pub type FilteredResults<'a> = Vec<&'a FindResult>;

struct ProcessedResults<'a> {
    results: FilteredResults<'a>,
    longest_path: usize,
    found_dirs: u64,
    found_bytes: u64,
    skipped_dirs: u64,
    skipped_bytes: u64,
}

pub fn find_command(command: &TreeWalk) -> Result<()> {
    if let Commands::Find(ref args) = &command.command {
        let mut results = FindResults::new();

        let from_path = Path::new(&command.from);

        do_find(from_path, &args.name, &mut results)?;

        let final_results = process_results(
            &results,
            command.min_mb,
            command.min_age_days,
            command.sort_by_age,
        );

        let path_width = final_results.longest_path;

        println!("");

        for result in final_results.results {
            println!(
                "| {:<path_width$} | uses {:>10} | {:4} days old |",
                result.path,
                ByteSize::b(result.size).to_string_as(true),
                result.days_old,
            );
        }

        println!(
            "\nFound   {:5} directories {}, total size {}",
            final_results.found_dirs,
            &args.name,
            ByteSize::b(final_results.found_bytes).to_string_as(true)
        );

        println!(
            "Skipped {:5} directories {}, total size {}",
            final_results.skipped_dirs,
            &args.name,
            ByteSize::b(final_results.skipped_bytes).to_string_as(true)
        );
    } else {
        anyhow::bail!("Incorrect args for 'find'");
    }

    Ok(())
}

fn process_results(
    all_results: &FindResults,
    min_mb: u64,
    min_age_days: u64,
    sort_by_age: bool,
) -> ProcessedResults {
    let sorted_results = sorted_results(&all_results, sort_by_age);

    // Filter results

    let mut found_dirs: u64 = 0;
    let mut found_bytes: u64 = 0;
    let mut skipped_dirs: u64 = 0;

    let mut filtered_results = Vec::new();

    for result in &sorted_results {
        if result.days_old >= min_age_days && result.size >= ByteSize::mb(min_mb).0 {
            found_dirs += 1;
            found_bytes += result.size;

            filtered_results.push(*result);
        } else {
            skipped_dirs += 1;
        }
    }

    let total_size = sorted_results.iter().fold(0, |t, e| t + e.size);

    ProcessedResults {
        longest_path: longest_path(&filtered_results),
        results: filtered_results,
        found_dirs,
        found_bytes,
        skipped_dirs,
        skipped_bytes: total_size - found_bytes,
    }
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

fn longest_path(results: &FilteredResults) -> usize {
    results
        .iter()
        .fold(0, |acc, result| usize::max(acc, result.path.len()))
}
