use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;
use std::{fs, os::unix::fs::MetadataExt};

use anyhow::Result;
use bytesize::ByteSize;
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "tree-walk")]
#[command(about = "Tree walking tool", long_about = None)]
struct TreeWalk {
    #[command(subcommand)]
    command: Commands,

    #[arg(global = true, long, default_value = ".")]
    from: String,

    #[arg(global = true, long, default_value_t = 10)]
    min_mb: u64,

    #[arg(global = true, long, default_value_t = 120)]
    min_age_days: u64,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Find(FindArgs),

    #[command(arg_required_else_help = true)]
    Delete(FindArgs),
}

#[derive(Debug, Args, Clone, PartialEq)]
struct CommonArgs {
    #[arg(default_value = ".")]
    from: String,
}

#[derive(Debug, Args)]
struct FindArgs {
    name: String,
}

fn main() -> Result<()> {
    let tree_walk = TreeWalk::parse();

    // println!("TreeWalk: {:?}", tree_walk);

    match tree_walk.command {
        Commands::Find(_) => {
            find_command(&tree_walk)?;
        }
        err => anyhow::bail!("Unknown command: {:?}", err),
    }

    Ok(())
}

pub struct FindResult {
    path: String,
    size: u64,
    days_old: u64,
}

type FindResults = HashMap<String, FindResult>;

fn find_command(command: &TreeWalk) -> Result<()> {
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

fn do_find(dir: &Path, name: &str, results: &mut FindResults) -> Result<()> {
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

fn days_old(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path)?;
    let now = SystemTime::now();
    let modified = now.duration_since(metadata.modified()?)?;
    let days_ago = modified.as_secs() / (24 * 3600);

    Ok(days_ago)
}

fn total_size(path: &Path) -> Result<u64> {
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
