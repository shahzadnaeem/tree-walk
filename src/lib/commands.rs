#[path = "commands/find.rs"]
pub mod find;

#[path = "commands/delete.rs"]
pub mod delete;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use delete::delete_command;
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
    #[command()]
    Find(FindArgs),

    #[command()]
    Delete(DeleteArgs),
}

const DEFAULT_NAME: &str = "node_modules";

#[derive(Debug, Args)]
pub struct FindArgs {
    #[arg(default_value = DEFAULT_NAME)]
    pub name: String,
}

impl From<&DeleteArgs> for FindArgs {
    fn from(value: &DeleteArgs) -> Self {
        FindArgs {
            name: value.name.clone(),
        }
    }
}

pub const NO_CODE: &str = "No code";

#[derive(Debug, Args)]
pub struct DeleteArgs {
    #[arg(default_value = DEFAULT_NAME)]
    pub name: String,
    #[arg(default_value = NO_CODE)]
    pub code: String,
}

impl TreeWalk {
    pub fn run() -> Result<()> {
        let tree_walk = TreeWalk::parse();

        match tree_walk.command {
            Commands::Find(_) => find_command(&tree_walk),
            Commands::Delete(_) => delete_command(&tree_walk),
        }
    }
}
