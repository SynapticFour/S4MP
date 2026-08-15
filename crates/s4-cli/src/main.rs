//! S4MP command-line interface.

mod commands;
mod graph_export;
mod workspace;

use clap::{Parser, Subcommand};
use commands::{
    analyze, certify, diff, graph, init, knowledge, map, plugin, query, reason, require, source,
    verify,
};
use s4_core::Result;

/// `SynapticFour` Method Platform CLI.
#[derive(Parser)]
#[command(
    name = "s4",
    version,
    about = "Heuristic Java↔Rust port maps (review → confirm → coverage)",
    long_about = "SynapticFour Method Platform (S4MP).\n\n\
Claimed use: compare a Java tree with a Rust port, review heuristic pairs, confirm them, \
then measure coverage. Shipped loop:\n\
  init → source add → graph build → map suggest → map show → map confirm → diff → verify → certify\n\n\
Satellite (not the port product): query, require, knowledge, plugin, reason.\n\n\
Honesty: certify evaluates verification-run policy only — not semantic equivalence. \
Default policy requires ≥1 manually confirmed Ported row.\n\n\
Maturity: heuristic-map-v2. Name (+ optional signature) similarity maps only."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new S4MP workspace (`.s4/` layout + metadata).
    Init {
        /// Target directory.
        #[arg(default_value = ".")]
        path: String,
    },
    /// Run the porting analysis pipeline (graph → map → diff).
    Analyze {
        /// Optional Java source alias (default: first registered Java source).
        #[arg(long)]
        java: Option<String>,
        /// Optional Rust source alias (default: first registered Rust source).
        #[arg(long)]
        rust: Option<String>,
        /// Diff report output path.
        #[arg(long, default_value = ".s4/reports/diff-report.md")]
        out: String,
        /// Rebuild graphs even when the physical snapshot hash is unchanged.
        #[arg(long, default_value_t = false)]
        force: bool,
        /// `git fetch` cached clones before building.
        #[arg(long, default_value_t = false)]
        refresh: bool,
    },
    /// Query a built source graph (`all` | `kind:callable` | `label~substr`).
    Query {
        /// Source alias (must have `graph build`).
        #[arg(long)]
        source: String,
        /// Query expression.
        #[arg(long, default_value = "all")]
        expr: String,
        /// Also print basic graph metrics.
        #[arg(long, default_value_t = false)]
        metrics: bool,
    },
    /// Run verification against porting/requirements thresholds.
    Verify {
        /// Java source alias.
        #[arg(long)]
        java: String,
        /// Rust source alias.
        #[arg(long)]
        rust: String,
        /// Minimum ported coverage percent (default 0 = always pass coverage).
        #[arg(long, default_value_t = 0.0)]
        min_coverage: f32,
        /// Fail when `MissingInTarget` rows remain.
        #[arg(long, default_value_t = false)]
        forbid_missing: bool,
        /// Fail when requirements exist without `ImplementedBy` traces.
        #[arg(long, default_value_t = false)]
        require_traced: bool,
    },
    /// Evaluate certification policy over a verification run.
    Certify {
        /// Policy name (`default`).
        #[arg(long, default_value = "default")]
        policy: String,
        /// Java source alias (selects verification run file).
        #[arg(long)]
        java: String,
        /// Rust source alias.
        #[arg(long)]
        rust: String,
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
    /// Requirements CRUD, `OpenAPI` import, and traces (satellite).
    #[command(hide = true)]
    Require {
        #[command(subcommand)]
        action: RequireAction,
    },
    /// Software knowledge graph helpers (satellite).
    #[command(hide = true)]
    Knowledge {
        #[command(subcommand)]
        action: KnowledgeAction,
    },
    /// List in-process plugins (Phase 6; WASM deferred).
    #[command(hide = true)]
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },
    /// Offline heuristic reasoning (outputs always Proposed).
    #[command(hide = true)]
    Reason {
        /// Intent: `explain` | `refactor` | `map` | `architecture`.
        #[arg(long, default_value = "explain")]
        intent: String,
        /// Optional prompt text hashed into the context bundle.
        #[arg(long)]
        prompt: Option<String>,
        /// Proposal JSON output path.
        #[arg(long, default_value = ".s4/proposals/latest.json")]
        out: String,
    },
}

#[derive(Subcommand)]
enum RequireAction {
    /// Add a requirement.
    Add {
        /// Requirement statement.
        statement: String,
        /// Kind: `functional` | `non_functional` | `constraint` | `test`.
        #[arg(long, default_value = "functional")]
        kind: String,
    },
    /// List requirements and traces.
    List,
    /// Import `OpenAPI` path keys as functional requirements.
    ImportOpenapi {
        /// Path to `OpenAPI` JSON document.
        path: String,
    },
    /// Suggest (and optionally apply) name-based requirement→callable traces.
    TraceSuggest {
        /// Built source alias.
        #[arg(long)]
        source: String,
        /// Persist suggestions.
        #[arg(long, default_value_t = false)]
        apply: bool,
    },
}

#[derive(Subcommand)]
enum KnowledgeAction {
    /// Extract naming concepts from a built graph.
    Extract {
        /// Built source alias.
        #[arg(long)]
        source: String,
    },
}

#[derive(Subcommand)]
enum PluginAction {
    /// List built-in registered plugins.
    List,
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
        /// Rebuild even when the physical snapshot hash is unchanged.
        #[arg(long, default_value_t = false)]
        force: bool,
        /// `git fetch` cached clones before building.
        #[arg(long, default_value_t = false)]
        refresh: bool,
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
    /// Diff two built graphs by `(kind, label)` identity.
    Diff {
        /// Left / baseline source alias.
        #[arg(long)]
        left: String,
        /// Right / candidate source alias.
        #[arg(long)]
        right: String,
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
        /// `git fetch` if this alias is already cached.
        #[arg(long, default_value_t = false)]
        refresh: bool,
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
    /// List correspondence rows (short id, pairing, signatures).
    Show {
        /// Java source alias (pass with `--rust` to select one map).
        #[arg(long)]
        java: Option<String>,
        /// Rust source alias.
        #[arg(long)]
        rust: Option<String>,
        /// Filter: `ported` | `diverged` | `missing` | `extra` | `unmapped`.
        #[arg(long)]
        status: Option<String>,
    },
    /// Confirm a correspondence as ported (`--id` or `--name`).
    Confirm {
        /// Correspondence id or unique prefix.
        #[arg(long)]
        id: Option<String>,
        /// Simple or qualified name (`add`, `Calculator.add`).
        #[arg(long)]
        name: Option<String>,
        /// Scope to this Java source alias.
        #[arg(long)]
        java: Option<String>,
        /// Scope to this Rust source alias.
        #[arg(long)]
        rust: Option<String>,
    },
    /// Reject a suggested correspondence (`--id` or `--name`).
    Reject {
        /// Correspondence id or unique prefix.
        #[arg(long)]
        id: Option<String>,
        /// Simple or qualified name (`add`, `Calculator.add`).
        #[arg(long)]
        name: Option<String>,
        /// Scope to this Java source alias.
        #[arg(long)]
        java: Option<String>,
        /// Scope to this Rust source alias.
        #[arg(long)]
        rust: Option<String>,
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
        Commands::Analyze {
            java,
            rust,
            out,
            force,
            refresh,
        } => analyze::run(java.as_deref(), rust.as_deref(), &out, force, refresh),
        Commands::Query {
            source,
            expr,
            metrics,
        } => query::run(&source, &expr, metrics),
        Commands::Verify {
            java,
            rust,
            min_coverage,
            forbid_missing,
            require_traced,
        } => verify::run(&java, &rust, min_coverage, forbid_missing, require_traced),
        Commands::Certify { policy, java, rust } => certify::run(&policy, &java, &rust),
        Commands::Source { action } => match action {
            SourceAction::Add {
                alias,
                git,
                local,
                git_ref,
                subpath,
                lang,
                refresh,
            } => source::run_add(
                &alias,
                git.as_deref(),
                local.as_deref(),
                git_ref.as_deref(),
                subpath.as_deref(),
                &lang,
                refresh,
            ),
            SourceAction::List => source::run_list(),
        },
        Commands::Graph { action } => match action {
            GraphAction::Build {
                source,
                out_dir,
                force,
                refresh,
            } => graph::run_build(&source, &out_dir, force, refresh),
            GraphAction::Export {
                source,
                format,
                filter,
                out,
            } => graph::run_export(&source, &format, &filter, &out),
            GraphAction::Diff { left, right } => graph::run_diff(&left, &right),
        },
        Commands::Map { action } => run_map(action),
        Commands::Diff { java, rust, out } => diff::run(&java, &rust, &out),
        Commands::Require { action } => match action {
            RequireAction::Add { statement, kind } => require::run_add(&kind, &statement),
            RequireAction::List => require::run_list(),
            RequireAction::ImportOpenapi { path } => require::run_import_openapi(&path),
            RequireAction::TraceSuggest { source, apply } => {
                require::run_trace_suggest(&source, apply)
            },
        },
        Commands::Knowledge { action } => match action {
            KnowledgeAction::Extract { source } => knowledge::run_extract(&source),
        },
        Commands::Plugin { action } => match action {
            PluginAction::List => plugin::run_list(),
        },
        Commands::Reason {
            intent,
            prompt,
            out,
        } => reason::run(&intent, prompt.as_deref(), &out),
    }
}

fn run_map(action: MapAction) -> Result<()> {
    match action {
        MapAction::Suggest { java, rust } => map::run_suggest(&java, &rust),
        MapAction::Show { java, rust, status } => {
            map::run_show(java.as_deref(), rust.as_deref(), status.as_deref())
        },
        MapAction::Confirm {
            id,
            name,
            java,
            rust,
        } => map::run_confirm(
            id.as_deref(),
            name.as_deref(),
            java.as_deref(),
            rust.as_deref(),
        ),
        MapAction::Reject {
            id,
            name,
            java,
            rust,
        } => map::run_reject(
            id.as_deref(),
            name.as_deref(),
            java.as_deref(),
            rust.as_deref(),
        ),
        MapAction::List => map::run_list(),
    }
}
