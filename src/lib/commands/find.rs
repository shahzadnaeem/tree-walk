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

        // Filter results

        let mut found_dirs: u64 = 0;
        let mut found_bytes: u64 = 0;
        let mut skipped_dirs: u64 = 0;

        for result in &results {
            if result.1.days_old >= command.min_age_days
                && result.1.size >= ByteSize::mb(command.min_mb).0
            {
                found_dirs += 1;
                found_bytes += result.1.size;

                println!(
                    "{:60} uses {}, {} days old",
                    result.1.path,
                    ByteSize::b(result.1.size),
                    result.1.days_old,
                );
            } else {
                skipped_dirs += 1;
            }
        }

        let total_size = results.iter().fold(0, |t, e| t + e.1.size);

        println!(
            "Found   {} directories {}, total size {}",
            found_dirs,
            &args.name,
            ByteSize::b(found_bytes)
        );

        println!(
            "Skipped {} directories {}, total size {}",
            skipped_dirs,
            &args.name,
            ByteSize::b(total_size - found_bytes)
        );
    } else {
        anyhow::bail!("Incorrect args for 'find'");
    }

    Ok(())
}
