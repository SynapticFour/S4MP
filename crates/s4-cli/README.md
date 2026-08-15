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
| `init [path]` | **implemented** | Create `.s4/` layout + `workspace.json` |
| `analyze` | **implemented** | Graph all sources → map/diff for Java+Rust pair |
| `verify` | **implemented** | Coverage/trace thresholds over artifacts |
| `certify --policy <name>` | **implemented** | Policy over `VerificationRun`. Default policy requires ≥1 **Ported** row — heuristic-only maps are `Invalid`. |
| `query --source <alias> --expr <expr>` | **implemented** | Filter query (`all` / `kind:*` / `label~*`) |
| `require …` / `knowledge` / `plugin` / `reason` | satellite | Hidden from `--help`; not the port-map product |

**Maturity:** `heuristic-map-v2`. Certification is **not** semantic equivalence. LLM/heuristic outputs are never ground truth.

```bash
s4 analyze --java mini-java --rust mini-rust
s4 map show --java mini-java --rust mini-rust
```


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

### Graph build & export

| Command | Description |
|---------|-------------|
| `graph build --source <alias> [--out-dir .s4/graphs] [--force] [--refresh]` | Parse source → USIR → semantic graph |
| `graph export --source <alias> [--format dot] [--filter …] [-o <file>]` | Export DOT or JSON (default: `.s4/exports/<alias>.{dot,json}`) |
| `graph diff --left <alias> --right <alias>` | Structural `(kind, label)` diff of two built graphs |

Pipeline steps for `graph build` (with terminal feedback):

1. Resolve source (clone; `git fetch` only with `--refresh`)
2. Physical snapshot → CAS (skip parse/lower when snapshot hash matches the last manifest, unless `--force`)
3. Tree-sitter parse → USIR modules (kept in memory; CAS reuse by file hash; also written to CAS)
4. Lower USIR → graph projection → CAS
5. Write graph manifest JSON

```bash
s4 graph build --source gatk-java-hc
s4 graph build --source hc-rust --out-dir .s4/graphs

# Visualize Rust graph (defaults write under .s4/exports/)
s4 graph export --source hc-rust --format dot --filter callable,calls,type,defines
dot -Tsvg .s4/exports/hc-rust.dot -o .s4/exports/hc-rust.svg
```

Makefile shortcuts: `make graph-export-rust`, `make graph-export-svg`

### Correspondence map

| Command | Description |
|---------|-------------|
| `map suggest --java <alias> --rust <alias>` | Heuristic Java→Rust correspondence (all `Diverged`) |
| `map show [--java --rust] [--status diverged]` | Row table: short id, pairing, signatures, status |
| `map confirm --id <prefix>` | Mark a **paired** row as Ported (unique prefix is enough) |
| `map confirm --name <symbol>` | Same, by simple or qualified name (errors if ambiguous) |
| `map reject --id` / `--name` | Drop a heuristic pairing (Java side becomes missing) |
| `map list` | List maps with ported/diverged/missing/extra counts |

```bash
s4 map suggest --java my-java --rust my-rust
s4 map show --java my-java --rust my-rust --status diverged
s4 map confirm --name add --java my-java --rust my-rust
s4 map confirm --id abcdef012345
s4 map list
```

### Diff report

| Command | Description |
|---------|-------------|
| `diff --java <alias> --rust <alias> [--out .s4/reports/diff-report.md]` | Render Markdown porting diff |

```bash
s4 diff --java gatk-java-hc --rust hc-rust
make open-report   # print default report path
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
| `store/<hex>.json` | CAS knowledge blobs |
| `graphs/<alias>.json` | `graph build` (workspace pointer) |
| `maps/<java>__<rust>.json` | `map suggest` (workspace pointer) |
| `reports/diff-report.md` (+ `.json` sidecar) | `diff` (default) |
| `exports/<alias>.dot` | `graph export` (default) |
| `verification/` | `verify` |
| `certificates/` | `certify` |
| `knowledge/` | `knowledge extract` |
| `proposals/` | `reason` |
| `requirements.json` | `require` |

Implementation: [`src/workspace.rs`](src/workspace.rs), command handlers in [`src/commands/`](src/commands/).

## Tier

4 — Surfaces
