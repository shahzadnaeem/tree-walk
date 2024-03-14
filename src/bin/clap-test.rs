use anyhow::Result;
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "clap-test")]
#[command(about = "A Test", long_about = None)]
pub struct ClapTest {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    // NOTE: Can omit arg_required_else_help = true as TestArgs has defaults
    #[command()]
    Test(TestArgs),
}

#[derive(Debug, Args)]
pub struct TestArgs {
    #[arg(default_value = "node_modules")]
    pub name: String,
    #[arg(default_value_t = 10_000_000)]
    pub min_size: usize,
}

fn main() -> Result<()> {
    let clap_test = ClapTest::parse();

    println!("We are doing this: {:?}", clap_test);

    Ok(())
}
