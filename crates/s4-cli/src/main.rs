//! S4MP command-line interface.

mod commands;
mod graph_export;
mod workspace;

use clap::{Parser, Subcommand};
use commands::{analyze, certify, diff, graph, init, map, query, source, verify};
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
    /// Register and manage source trees.
    Source {
        #[command(subcommand)]
        action: SourceAction,
    },
    /// Build or export semantic graphs for registered sources.
    Graph {
        #[command(subcommand)]
        action: GraphAction,
    },
    /// Manage Java↔Rust correspondence maps.
    Map {
        #[command(subcommand)]
        action: MapAction,
    },
    /// Render a Markdown porting diff report.
    Diff {
        /// Java source alias.
        #[arg(long)]
        java: String,
        /// Rust source alias.
        #[arg(long)]
        rust: String,
        /// Output Markdown file path.
        #[arg(long, default_value = ".s4/reports/diff-report.md")]
        out: String,
    },
}

#[derive(Subcommand)]
enum GraphAction {
    /// Parse source, lower USIR, and write graph manifest + CAS artifacts.
    Build {
        /// Source alias from `source add`.
        #[arg(long)]
        source: String,
        /// Directory for graph manifest output.
        #[arg(long, default_value = ".s4/graphs")]
        out_dir: String,
    },
    /// Export a built graph for visualization (Graphviz DOT or JSON).
    Export {
        /// Source alias (must have been built with `graph build`).
        #[arg(long)]
        source: String,
        /// Output format: `dot` or `json`.
        #[arg(long, default_value = "dot")]
        format: String,
        /// Comma-separated node/edge kinds (`callable,calls,type,defines`, or `all`).
        #[arg(long, default_value = "callable,calls,type,defines")]
        filter: String,
        /// Output file path.
        #[arg(long, short = 'o', default_value = ".s4/exports/graph")]
        out: String,
    },
}

#[derive(Subcommand)]
enum SourceAction {
    /// Register a Git or local source tree.
    Add {
        /// Short alias for the source (e.g. `gatk-java-hc`).
        alias: String,
        /// Git clone URL.
        #[arg(long)]
        git: Option<String>,
        /// Local filesystem path.
        #[arg(long)]
        local: Option<String>,
        /// Git branch, tag, or commit.
        #[arg(long)]
        git_ref: Option<String>,
        /// Subdirectory within the Git repository.
        #[arg(long)]
        subpath: Option<String>,
        /// Primary language (`java` or `rust`).
        #[arg(long)]
        lang: String,
    },
    /// List registered sources.
    List,
}

#[derive(Subcommand)]
enum MapAction {
    /// Suggest correspondences between Java and Rust graphs.
    Suggest {
        /// Java source alias.
        #[arg(long)]
        java: String,
        /// Rust source alias.
        #[arg(long)]
        rust: String,
    },
    /// Confirm a correspondence as ported.
    Confirm {
        /// Correspondence entry id.
        #[arg(long)]
        id: String,
    },
    /// Reject a suggested correspondence.
    Reject {
        /// Correspondence entry id.
        #[arg(long)]
        id: String,
    },
    /// List correspondence maps.
    List,
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
        Commands::Source { action } => match action {
            SourceAction::Add {
                alias,
                git,
                local,
                git_ref,
                subpath,
                lang,
            } => source::run_add(
                &alias,
                git.as_deref(),
                local.as_deref(),
                git_ref.as_deref(),
                subpath.as_deref(),
                &lang,
            ),
            SourceAction::List => source::run_list(),
        },
        Commands::Graph { action } => match action {
            GraphAction::Build { source, out_dir } => graph::run_build(&source, &out_dir),
            GraphAction::Export {
                source,
                format,
                filter,
                out,
            } => graph::run_export(&source, &format, &filter, &out),
        },
        Commands::Map { action } => match action {
            MapAction::Suggest { java, rust } => map::run_suggest(&java, &rust),
            MapAction::Confirm { id } => map::run_confirm(&id),
            MapAction::Reject { id } => map::run_reject(&id),
            MapAction::List => map::run_list(),
        },
        Commands::Diff { java, rust, out } => diff::run(&java, &rust, &out),
    }
}
