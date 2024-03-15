use std::{fs::remove_dir_all, io::Write, thread::sleep, time::Duration};

use anyhow::Result;
use bytesize::ByteSize;
use totp_rs::{Algorithm, Secret, TOTP};

use super::{Commands, FindArgs, TreeWalk, NO_CODE};
use crate::utils::find;

pub fn delete_command(command: &TreeWalk) -> Result<()> {
    if let Commands::Delete(ref args) = &command.command {
        let find_args = FindArgs::from(args);
        let results = find(command, &find_args)?;

        let path_width = results.longest_path;

        if results.found_dirs > 0 {
            println!("Found the following directories to DELETE\n");

            for result in &results.results {
                println!(
                    "| {:<path_width$} | uses {:>10} | {:4} days old |",
                    result.path,
                    ByteSize::b(result.size).to_string_as(true),
                    result.days_old,
                );
            }

            let can_proceed = check_code(&args.code);

            println!("\nDeletion will begin in 5 seconds. Are you SURE?");
            countdown_timer(5);
            println!("");

            if can_proceed {
                for result in &results.results {
                    println!("Deleting {} ...", result.path);
                    if let Err(err) = remove_dir_all(&result.path) {
                        eprintln!("Unable to delete {} - {}", &result.path, err);
                    }
                }
            } else {
                println!("Sorry, missing or invalid confirmation code");
            }
        } else {
            println!("Nothing to delete!");
        }
    } else {
        anyhow::bail!("Incorrect args for 'delete'");
    }

    Ok(())
}

fn check_code(code: &String) -> bool {
    if code == NO_CODE {
        println!(
            "Please supply the following code to confirm you wish to proceed: {}",
            current_code()
        );
        false
    } else {
        code == &current_code()
    }
}

fn current_code() -> String {
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Raw("RandomUnimportantSecret".as_bytes().to_vec())
            .to_bytes()
            .unwrap(),
    )
    .unwrap();
    let token = totp.generate_current().unwrap();

    token
}

fn countdown_timer(secs: u32) {
    for sec in 1..=secs {
        print!("\r{}...", secs - sec);
        std::io::stdout().flush().unwrap_or(());
        sleep(Duration::from_secs(1));
    }
}
