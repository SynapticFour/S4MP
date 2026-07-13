//! S4MP command-line interface.

mod commands;

use clap::{Parser, Subcommand};
use commands::{analyze, init, query};

#[derive(Parser)]
#[command(name = "s4mp", about = "SynapticFour Method Platform")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new S4MP workspace
    Init {
        #[arg(default_value = ".")]
        path: String,
    },
    /// Run analysis pipeline
    Analyze,
    /// Query the knowledge graph
    Query {
        #[arg(long, default_value = "all")]
        expr: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Init { path } => init::run(&path),
        Commands::Analyze => analyze::run(),
        Commands::Query { expr } => query::run(&expr),
    };
    if let Err(err) = result {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
