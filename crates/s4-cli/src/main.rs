//! S4MP command-line interface.

mod commands;

use clap::{Parser, Subcommand};
use commands::{analyze, certify, init, query, verify};
use s4_core::Result;

/// `SynapticFour` Method Platform CLI.
#[derive(Parser)]
#[command(name = "s4", version, about = "SynapticFour Method Platform")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new S4MP workspace.
    Init {
        /// Target directory.
        #[arg(default_value = ".")]
        path: String,
    },
    /// Run the analysis pipeline.
    Analyze,
    /// Query the knowledge graph.
    Query {
        /// Query expression.
        #[arg(long, default_value = "all")]
        expr: String,
    },
    /// Run verification against invariants.
    Verify,
    /// Evaluate certification policy.
    Certify {
        /// Policy name.
        #[arg(long, default_value = "default")]
        policy: String,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

/// Dispatch CLI subcommands.
///
/// # Errors
///
/// Returns an error if a subcommand fails.
fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init { path } => init::run(&path),
        Commands::Analyze => analyze::run(),
        Commands::Query { expr } => query::run(&expr),
        Commands::Verify => verify::run(),
        Commands::Certify { policy } => certify::run(&policy),
    }
}
