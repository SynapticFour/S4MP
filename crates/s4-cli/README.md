# s4-cli

Command-line interface for the SynapticFour Method Platform.

## Binary

`s4` — primary CLI entry point (`cargo run -p s4-cli -- …`).

## Getting Started

See the **[Porting Workflow Guide](../../docs/guides/PORTING_WORKFLOW.md)** for a beginner walkthrough (Makefile + manual CLI).

Quick smoke test:

```bash
cargo run -p s4-cli -- --help
cargo run -p s4-cli -- source list
```

## Subcommands

### Workspace

| Command | Status | Description |
|---------|--------|-------------|
| `init [path]` | stub | Initialize workspace |
| `analyze` | stub | Run analysis pipeline |
| `query --expr <expr>` | stub | Query knowledge graph |
| `verify` | stub | Run verification |
| `certify --policy <name>` | stub | Evaluate certification policy |

### Source management

| Command | Description |
|---------|-------------|
| `source add <alias> --git <url> --lang java [--subpath …] [--git-ref …]` | Register a Git source |
| `source add <alias> --local <path> --lang rust` | Register a local source |
| `source list` | List registered sources |

Examples:

```bash
s4 source add gatk-java-hc \
  --git https://github.com/broadinstitute/gatk.git \
  --subpath src/main/java/.../haplotypecaller \
  --lang java

s4 source add hc-rust --local ../my-hc-port --lang rust
s4 source list
```

### Graph build

| Command | Description |
|---------|-------------|
| `graph --source <alias> [--out-dir .s4/graphs]` | Parse source → USIR → semantic graph |

Pipeline steps (with terminal feedback):

1. Resolve source (clone/fetch for Git)
2. Physical snapshot → CAS
3. Tree-sitter parse → USIR modules → CAS
4. Lower USIR → graph projection → CAS
5. Write graph manifest JSON

```bash
s4 graph --source gatk-java-hc
s4 graph --source hc-rust --out-dir .s4/graphs
```

### Correspondence map

| Command | Description |
|---------|-------------|
| `map suggest --java <alias> --rust <alias>` | Heuristic Java→Rust correspondence |
| `map confirm --id <entry-id>` | Mark entry as manually ported |
| `map reject --id <entry-id>` | Reject a heuristic pairing |
| `map list` | List all correspondence maps |

```bash
s4 map suggest --java gatk-java-hc --rust hc-rust
s4 map confirm --id abc123…
s4 map list
```

### Diff report

| Command | Description |
|---------|-------------|
| `diff --java <alias> --rust <alias> [--out diff-report.md]` | Render Markdown porting diff |

```bash
s4 diff --java gatk-java-hc --rust hc-rust --out diff-report.md
```

## Makefile Shortcuts

The repository root [`Makefile`](../../Makefile) wraps the porting pipeline:

```bash
make sources RUST_LOCAL=../my-hc-port
make diff
```

See [Porting Workflow Guide](../../docs/guides/PORTING_WORKFLOW.md) for all targets and variables.

## Workspace Files

The CLI reads/writes under `.s4/` in the current working directory:

| Path | Written by |
|------|------------|
| `sources.json` | `source add` |
| `cache/<alias>/` | Git ingest |
| `store/<hex>.json` | All artifact writes |
| `graphs/<alias>.json` | `graph` |
| `maps/<java>__<rust>.json` | `map suggest` |

Implementation: [`src/workspace.rs`](src/workspace.rs), command handlers in [`src/commands/`](src/commands/).

## Tier

4 — Surfaces
