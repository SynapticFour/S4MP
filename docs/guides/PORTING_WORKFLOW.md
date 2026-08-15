# Java→Rust Porting Workflow (v1)

This guide walks through the **intended loop**: register two trees, build name graphs, review heuristic pairs **with ids**, confirm them, render a diff, then verify/certify coverage.

It is aimed at beginners — no prior S4MP knowledge required beyond a working Rust toolchain.

## What You Get

| Step | Command | Output |
|------|---------|--------|
| Register sources | `s4 source add` | `.s4/sources.json` |
| Build graphs | `s4 graph build` | USIR + graph in CAS, manifest in `.s4/graphs/` |
| Suggest mappings | `s4 map suggest` | Correspondence map in `.s4/maps/` (all pairs are `Diverged`) |
| Review rows | `s4 map show` | Short id, Java↔Rust pairing, signatures, status |
| Confirm a pair | `s4 map confirm --id` / `--name` | That row becomes `Ported` |
| Diff report | `s4 diff` | `.s4/reports/diff-report.md` with `id=` on every row |
| Coverage check | `s4 verify` / `s4 certify` | Thresholds over counters — **not** semantic equivalence |

Default `s4 certify` is Valid only with **≥1 Ported** row. Heuristic-only maps write `Invalid` and exit non-zero.

## Prerequisites

```bash
# Rust toolchain (see rust-version in Cargo.toml)
rustup toolchain install stable --component rustfmt clippy

# Git (for cloning Java sources)
git --version

# Optional: make (for the bundled Makefile)
make --version
```

Build the workspace once:

```bash
cargo build --workspace
```

## Workspace Layout

Running commands from the repository root creates a local `.s4/` directory (git-ignored):

```
.s4/
  sources.json              # registered source aliases
  cache/<alias>/            # git clones (sparse checkout when --subpath is set)
  store/<hex>.json          # content-addressed artifacts (USIR, graphs, maps)
  graphs/<alias>.json       # graph build manifests
  maps/<java>__<rust>.json  # correspondence map manifests
  reports/                  # diff reports (default: diff-report.md)
  exports/                  # graph DOT/JSON/SVG exports
```

## Option A — Fixture (recommended first run)

No network. Uses `tests/fixtures/mini-port/`.

```bash
make e2e-fixture
```

Or by hand against your own copies of those trees (see README “Your own trees”). After `map suggest`:

```bash
cargo run -p s4-cli -- map show --java mini-java --rust mini-rust --status diverged
cargo run -p s4-cli -- map confirm --name add --java mini-java --rust mini-rust
cargo run -p s4-cli -- diff --java mini-java --rust mini-rust
cargo run -p s4-cli -- verify --java mini-java --rust mini-rust
cargo run -p s4-cli -- certify --java mini-java --rust mini-rust
```

`map show` prints a 12-character id prefix. `map confirm --id` accepts that prefix when it is unique. `--name add` fails if two rows share that simple name — use `--id` or a qualified name (`Calculator.add`).

You cannot confirm `ExtraInTarget` (Rust-only) or `MissingInTarget` (Java-only) rows. Those stay extras/missing until the other side exists or you reject a bad pairing.

## Option B — Makefile (GATK HaplotypeCaller slice)

The root [`Makefile`](../../Makefile) wraps the full pipeline with sensible defaults for the GATK HaplotypeCaller Java slice:

| Variable | Default | Purpose |
|----------|---------|---------|
| `JAVA_ALIAS` | `gatk-java-hc` | Alias for the Java source |
| `JAVA_GIT` | GATK GitHub URL | Clone URL |
| `JAVA_SUBPATH` | `.../haplotypecaller` | Subdirectory within the repo |
| `RUST_ALIAS` | `hc-rust` | Alias for your Rust port |
| `RUST_LOCAL` | `../my-hc-port` | Path to local Rust checkout |

### Step-by-step

```bash
# 1. Register Java (git) + Rust (local) sources
make sources RUST_LOCAL=../my-hc-port

# 2. Build both semantic graphs (clone + parse + lower)
make graph

# 3. Suggest Java↔Rust correspondences
make map

# 4. Review rows, then confirm at least one pair
make show
cargo run -p s4-cli -- map confirm --name <symbol> --java gatk-java-hc --rust hc-rust

# 5. Render Markdown diff report
make diff
# → .s4/reports/diff-report.md
make open-report   # print report path

# 6. Coverage / policy (Valid only after a confirm)
make verify
make certify
```

| `GRAPH_FILTER` | `callable,calls,type,defines` | Node/edge kinds for `graph export` |

Override any variable inline:

```bash
make graph JAVA_SUBPATH=src/main/java/org/broadinstitute/hellbender/tools/walkers/haplotypecaller
make diff RUST_LOCAL=../other-port
make graph-export-rust GRAPH_FILTER=callable,calls
make graph-export-svg
```

Reset cached artifacts (keeps `sources.json` and map manifests):

```bash
make clean-cache
```

### Makefile targets

| Target | Depends on | Action |
|--------|------------|--------|
| `sources` | — | `s4 source add` for Java + Rust |
| `graph-java` | — | Build graph for Java alias |
| `graph-rust` | — | Build graph for Rust alias |
| `graph` | `graph-java`, `graph-rust` | Both graphs |
| `graph-export` | `graph-export-java`, `graph-export-rust` | Export DOT files to `.s4/exports/` |
| `graph-export-svg` | `graph-export-rust` | Render `.s4/exports/hc-rust.svg` (requires Graphviz) |
| `map` | `graph` | `s4 map suggest` |
| `show` | `map` | `s4 map show` (short ids + pairings) |
| `diff` | `map` | `s4 diff` → `.s4/reports/diff-report.md` |
| `verify` | `map` | `s4 verify` (coverage thresholds) |
| `certify` | `verify` | `s4 certify` (fails until ≥1 Ported row) |
| `e2e-fixture` | — | Workspace tests for the mini-port review loop |
| `open-report` | — | Print diff report path |
| `install-hooks` | — | Enable local pre-commit checks (fmt, clippy, test) |
| `clean-cache` | — | Remove `.s4/cache`, `.s4/store`, `.s4/graphs` |

## Option C — CLI Only

All Makefile steps map directly to `cargo run` invocations.

### 1. Register sources

**Java (Git + subpath):**

```bash
cargo run -p s4-cli -- source add gatk-java-hc \
  --git https://github.com/broadinstitute/gatk.git \
  --subpath src/main/java/org/broadinstitute/hellbender/tools/walkers/haplotypecaller \
  --lang java
```

**Rust (local directory):**

```bash
cargo run -p s4-cli -- source add hc-rust \
  --local ../my-hc-port \
  --lang rust
```

Optional flags for Git sources:

- `--git-ref <branch|tag|commit>` — pin to a specific ref
- `--subpath <dir>` — limit scope to a subdirectory

List registered sources:

```bash
cargo run -p s4-cli -- source list
```

### 2. Build semantic graphs

```bash
cargo run -p s4-cli -- graph build --source gatk-java-hc
cargo run -p s4-cli -- graph build --source hc-rust
```

Optional: custom manifest output directory (default: `.s4/graphs`):

```bash
cargo run -p s4-cli -- graph build --source gatk-java-hc --out-dir .s4/graphs
```

Expected terminal output includes file counts, callable/type statistics, and artifact IDs.

### 2b. Export graph for visualization (optional)

Export a Graphviz DOT file (after `graph build`):

```bash
cargo run -p s4-cli -- graph export \
  --source hc-rust \
  --format dot \
  --filter callable,calls,type,defines
```

Default output: `.s4/exports/hc-rust.dot` (one file per source alias).

Render SVG (requires [Graphviz](https://graphviz.org/)):

```bash
dot -Tsvg .s4/exports/hc-rust.dot -o .s4/exports/hc-rust.svg
open .s4/exports/hc-rust.svg
```

Or with Make:

```bash
make graph-export-rust   # build + export .s4/exports/hc-rust.dot
make graph-export-svg    # also render .s4/exports/hc-rust.svg
```

**Filter tokens** (comma-separated, or `all`):

| Token | Matches |
|-------|---------|
| `callable`, `type`, `module`, `symbol`, `package` | Node kinds |
| `calls`, `defines`, `references`, `depends_on`, `implements` | Edge kinds |

Edges include their endpoint nodes even when the endpoint kind is not listed (e.g. `callable,calls` shows call edges between callables).

JSON export: `--format json` → `.s4/exports/hc-rust.json` (default naming)

### 3. Suggest correspondences

```bash
cargo run -p s4-cli -- map suggest --java gatk-java-hc --rust hc-rust
```

List maps (counts only) or **rows** (what you actually review):

```bash
cargo run -p s4-cli -- map list
cargo run -p s4-cli -- map show --java gatk-java-hc --rust hc-rust
cargo run -p s4-cli -- map show --java gatk-java-hc --rust hc-rust --status diverged
```

### 4. Manual review (required for certify)

Confirm a suggested mapping from `map show` or the diff report (`id=`):

```bash
cargo run -p s4-cli -- map confirm --id <12-char-prefix>
cargo run -p s4-cli -- map confirm --name add --java gatk-java-hc --rust hc-rust
```

Reject a heuristic pairing (becomes Missing in Rust):

```bash
cargo run -p s4-cli -- map reject --id <12-char-prefix>
```

Re-run `map suggest` after graph rebuilds; manual confirmations are preserved via `merge_correspondences`.

### 5. Render diff report

```bash
cargo run -p s4-cli -- diff \
  --java gatk-java-hc \
  --rust hc-rust
```

Default output: `.s4/reports/diff-report.md`. Override with `--out <path>`.

Open `.s4/reports/diff-report.md` in your editor. Sections:

- **How to review** — `map show` / `map confirm` commands
- **Summary** — coverage metrics
- **Missing in Rust** — Java nodes with no Rust counterpart (`id=`)
- **Diverged (review these)** — heuristic pairs with Java↔Rust names, signatures, confidence, `id=`
- **Extra in Rust** — Rust-only nodes
- **Ported (manually confirmed)** — rows you confirmed

### 6. Verify and certify (coverage only)

```bash
cargo run -p s4-cli -- verify --java gatk-java-hc --rust hc-rust
cargo run -p s4-cli -- certify --java gatk-java-hc --rust hc-rust
```

Without a confirmed Ported row, certify writes `Invalid` and exits non-zero.

## Pipeline Internals (Brief)

```
SourceRef  →  DefaultSourceIngestor  →  local tree
                ↓
         snapshot_physical  →  PhysicalSnapshot artifact
                ↓
         JavaParser / RustParser  →  UsirModule artifacts
                ↓
         usir_to_graph  →  GraphProjection artifact
                ↓
         suggest_correspondences  →  CorrespondenceMap artifact
                ↓
         build_diff_report + render_markdown  →  .s4/reports/diff-report.md
```

Key crates:

| Crate | Role |
|-------|------|
| [`s4-project`](../../crates/s4-project/README.md) | Source registration, git clone cache, snapshots |
| [`s4-parser`](../../crates/s4-parser/README.md) | Tree-sitter Java/Rust → USIR |
| [`s4-graph`](../../crates/s4-graph/README.md) | In-memory graph view |
| [`s4-analysis`](../../crates/s4-analysis/README.md) | Lowering, correspondence, diff report |
| [`s4-storage`](../../crates/s4-storage/README.md) | `FileSystemStore` CAS |
| [`s4-cli`](../../crates/s4-cli/README.md) | CLI orchestration |

## Operator limits

These are product rules, not bugs:

| Limit | What happens |
|-------|----------------|
| `--name` matches two rows | Error listing both short ids — use `--id` or a qualified name (`Calculator.add`) |
| Confirm extra (Rust-only) or missing (Java-only) | Error — those rows are not a pair |
| Re-run `map suggest` after confirms | **Manual** `Ported` rows are kept; heuristic `Diverged` rows are replaced |
| `make sources` twice with the same alias | Fails — register once |
| `s4 certify` with zero Ported rows | Writes `Invalid`, exit non-zero |
| `s4 certify` Valid | Policy over counters only — **not** “the port is semantically correct” |
| One maintainer | No review roster; availability is best-effort |

There is no crates.io release. Optional PATH install from this checkout: `cargo install --path crates/s4-cli --locked`.

## Limitations (v1)

- **Name heuristics only** — no semantic or type-system equivalence. Signatures are Jaccard tokens, not a type checker.
- **Heuristic suggestions are `Diverged`**, never auto-`Ported`.
- **Call edges** come from Tree-sitter `method_invocation` / `call_expression` nodes; unresolved names are linked across files by simple callee name when unique.
- **Re-running `source add` with the same alias fails** — edit `.s4/sources.json` or use a new alias.
- **`make sources` is not idempotent** — register once, then use `graph` / `map` / `show` / `diff`.

## Troubleshooting

| Problem | Likely cause | Fix |
|---------|--------------|-----|
| `unknown source alias` | Source not registered | Run `source add` or `make sources` |
| `graph manifest ... not found` | Graph not built | Run `s4 graph build --source <alias>` |
| `git clone failed` | Network / URL / ref | Check `--git`, `--git-ref`; try manual clone into `.s4/cache/<alias>` |
| `local source path does not exist` | Wrong `--local` path | Use absolute or correct relative path |
| Empty diff / 0% coverage | Nothing confirmed as Ported | `s4 map show` then `s4 map confirm --name …` |
| `cannot confirm extra-in-target` | Rust-only symbol | Leave it extra, or add the Java counterpart |
| `ambiguous --name` | Two rows share that identifier | Use `--id` from `map show` |
| Slow first run | Git shallow/sparse clone + parse | Subsequent runs reuse `.s4/cache/` |

## Next Steps

- Read [Engineering Standards](../engineering/ENGINEERING_STANDARDS.md) before extending the pipeline.
- See [ADR-013: LLVM infrastructure model](../adr/0013-llvm-infrastructure-not-sonarqube.md) for design rationale.
- Per-crate APIs: `crates/<name>/README.md`.
