#[path = "commands/find.rs"]
pub mod find;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use find::find_command;

#[derive(Debug, Parser)]
#[command(name = "tree-walk")]
#[command(about = "Tree walking tool", long_about = None)]
pub struct TreeWalk {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(global = true, long, default_value = ".")]
    pub from: String,

    #[arg(global = true, long, default_value_t = 10)]
    pub min_mb: u64,

    #[arg(global = true, long, default_value_t = 14)]
    pub min_age_days: u64,

    #[arg(global = true, long, default_value_t = false)]
    pub sort_by_age: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(arg_required_else_help = true)]
    Find(FindArgs),

    #[command(arg_required_else_help = true)]
    Delete(FindArgs),
}

#[derive(Debug, Args)]
pub struct FindArgs {
    pub name: String,
}

impl TreeWalk {
    pub fn run() -> Result<()> {
        let tree_walk = TreeWalk::parse();

        match tree_walk.command {
            Commands::Find(_) => find_command(&tree_walk),
            err => anyhow::bail!("Unknown command: {:?}", err),
        }
    }
}
