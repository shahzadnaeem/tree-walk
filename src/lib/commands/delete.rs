use std::{fs::remove_dir_all, io::Write, thread::sleep, time::Duration};

use anyhow::Result;
use bytesize::ByteSize;

use super::{Commands, TreeWalk};
use crate::utils::find;

pub fn delete_command(command: &TreeWalk) -> Result<()> {
    if let Commands::Delete(ref args) = &command.command {
        let results = find(command, args)?;

        let path_width = results.longest_path;

        if results.found_dirs > 0 {
            println!("Found the following directories to DELETE!...\n");

            for result in &results.results {
                println!(
                    "| {:<path_width$} | uses {:>10} | {:4} days old |",
                    result.path,
                    ByteSize::b(result.size).to_string_as(true),
                    result.days_old,
                );
            }

            println!("\nDeletion will begin in 5 seconds...");
            countdown_timer(5);
            println!("");

            for result in &results.results {
                println!("Deleting {}!", result.path);
                if let Err(err) = remove_dir_all(&result.path) {
                    eprintln!("Unable to delete {} - {}", &result.path, err);
                }
            }
        } else {
            println!("Nothing to delete!");
        }
    } else {
        anyhow::bail!("Incorrect args for 'delete'");
    }

    Ok(())
}

fn countdown_timer(secs: u32) {
    for sec in 1..=secs {
        print!("\r{}...", secs - sec);
        std::io::stdout().flush().unwrap_or(());
        sleep(Duration::from_secs(1));
    }
}
