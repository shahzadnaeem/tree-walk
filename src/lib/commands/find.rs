use anyhow::Result;
use bytesize::ByteSize;

use super::{Commands, TreeWalk};
use crate::utils::find;

pub fn find_command(command: &TreeWalk) -> Result<()> {
    if let Commands::Find(ref args) = &command.command {
        let results = find(command, args)?;

        let path_width = results.longest_path;

        println!("");

        for result in results.results {
            println!(
                "| {:<path_width$} | uses {:>10} | {:4} days old |",
                result.path,
                ByteSize::b(result.size).to_string_as(true),
                result.days_old,
            );
        }

        println!(
            "\nFound   {:5} directories {}, total size {}",
            results.found_dirs,
            &args.name,
            ByteSize::b(results.found_bytes).to_string_as(true)
        );

        println!(
            "Skipped {:5} directories {}, total size {}",
            results.skipped_dirs,
            &args.name,
            ByteSize::b(results.skipped_bytes).to_string_as(true)
        );
    } else {
        anyhow::bail!("Incorrect args for 'find'");
    }

    Ok(())
}
