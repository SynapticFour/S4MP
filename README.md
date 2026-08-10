# SynapticFour Method Platform (S4MP)

A production-grade, modular, plugin-driven platform for software knowledge extraction, analysis, and certification.

**The knowledge model is the product. AI is one consumer among many.**

> **Maturity:** `heuristic-map-v2`
> What ships today is a **heuristic Java↔Rust port map** (`source` → `graph` → `map` → `diff`) using name (+ optional signature) similarity.
> It is **not** semantic equivalence, **not** a certificate, and `s4 certify` / `s4 verify` are **not implemented**.
> Roadmap: [Implementation Roadmap](docs/guides/IMPLEMENTATION_ROADMAP.md).

## Quick Start (Porting Pipeline)

The fastest way to compare a Java codebase with a Rust port and get a Markdown diff report:

### Prerequisites

- Rust stable (`rustfmt`, `clippy`)
- Git (for cloning Java sources)
- `make` (optional, recommended)

```bash
cargo build --workspace
```

### Using the Makefile

From the repository root, with your Rust port at `../my-hc-port`:

```bash
make sources RUST_LOCAL=../my-hc-port   # register GATK HC slice + Rust port
make diff                               # graph → map → .s4/reports/diff-report.md
```

Defaults target the **GATK HaplotypeCaller** Java slice. Override any variable:

```bash
make graph JAVA_SUBPATH=src/main/java/org/broadinstitute/hellbender/tools/walkers/haplotypecaller
make diff RUST_LOCAL=../other-port
```

See [`Makefile`](Makefile) for all targets (`sources`, `graph`, `graph-export`, `graph-export-svg`, `map`, `diff`, `open-report`, `install-hooks`, `clean-cache`).

### Using the CLI directly

```bash
# Register sources
cargo run -p s4-cli -- source add gatk-java-hc \
  --git https://github.com/broadinstitute/gatk.git \
  --subpath src/main/java/org/broadinstitute/hellbender/tools/walkers/haplotypecaller \
  --lang java
cargo run -p s4-cli -- source add hc-rust --local ../my-hc-port --lang rust

# Build graphs, suggest mappings, render report
cargo run -p s4-cli -- graph build --source gatk-java-hc
cargo run -p s4-cli -- graph build --source hc-rust
cargo run -p s4-cli -- map suggest --java gatk-java-hc --rust hc-rust
cargo run -p s4-cli -- diff --java gatk-java-hc --rust hc-rust

# Visualize Rust graph (requires graph build + Graphviz for SVG)
cargo run -p s4-cli -- graph export --source hc-rust --format dot --filter callable,calls,type,defines
dot -Tsvg .s4/exports/hc-rust.dot -o .s4/exports/hc-rust.svg
```

Or with Make:

```bash
make graph-export-rust    # builds graph + exports .s4/exports/hc-rust.dot
make graph-export-svg     # also renders .s4/exports/hc-rust.svg (needs `dot`)
make open-report          # print path to diff report
```

**Full beginner guide:** [Porting Workflow](docs/guides/PORTING_WORKFLOW.md) — step-by-step CLI reference, troubleshooting, pipeline internals.

**CLI command reference:** [`crates/s4-cli/README.md`](crates/s4-cli/README.md)

### Workspace artifacts

Commands create a local `.s4/` directory (git-ignored):

```
.s4/
  sources.json       # registered aliases
  cache/             # git clones (sparse checkout when --subpath is set)
  store/             # content-addressed JSON artifacts
  graphs/            # graph build manifests
  maps/              # correspondence map manifests
  reports/           # diff reports (default: diff-report.md)
  exports/           # graph DOT/JSON/SVG exports
```

## Workspace

This repository is a [Cargo workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html) of 18 focused crates. Each crate owns a single concern, exposes a documented public API of traits and types, and compiles independently.

```
crates/
  s4-core            Foundation: IDs, errors, versioning
  s4-storage         Content-addressed artifact store
  s4-events          Event bus contracts
  s4-plugin          Plugin system contracts
  s4-project         Project workspace & source ingestion
  s4-parser          Universal parsing & USIR (Java/Rust v1)
  s4-graph           Universal code graph
  s4-knowledge       Software knowledge graph contracts
  s4-requirements    Requirements graph & traceability
  s4-metrics         Complexity & metrics contracts
  s4-analysis        Lowering, correspondence, diff reports
  s4-planner         Refactoring planning contracts
  s4-verification    Verification & acceptance workflows
  s4-certification   Certification & compliance
  s4-llm             LLM-agnostic reasoning (interfaces only)
  s4-api             HTTP/gRPC API contracts
  s4-cli             Command-line interface (`s4`)
  s4-ui              UI integration contracts
```

## Dependency Tiers

Dependency direction is strictly **inward**:

```
Surfaces (s4-cli, s4-api, s4-ui)
    ↓
Quality (s4-verification, s4-certification, s4-planner)
    ↓
Capabilities (s4-parser, s4-metrics, s4-analysis, s4-llm, s4-requirements)
    ↓
Knowledge (s4-knowledge, s4-graph)
    ↓
Infrastructure (s4-storage, s4-events, s4-plugin, s4-project)
    ↓
Foundation (s4-core)
```

## Development

After cloning, run once to enable local pre-commit checks (fmt, clippy, test):

```bash
make install-hooks
```

This sets `core.hooksPath` to `.githooks/` in your **local** Git config (not versioned). Each contributor must run `make install-hooks` themselves — the repository cannot enforce this for everyone automatically.

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

See [Contributing](CONTRIBUTING.md) and [Engineering Standards](docs/engineering/ENGINEERING_STANDARDS.md).

## Documentation

### Guides

- **[Implementation Roadmap](docs/guides/IMPLEMENTATION_ROADMAP.md)** — phased delivery (P0–P6)
- **[Porting Workflow](docs/guides/PORTING_WORKFLOW.md)** — Java→Rust diff pipeline (Makefile + CLI)

### Architecture & Standards

- [Engineering Standards](docs/engineering/ENGINEERING_STANDARDS.md) — **mandatory before implementation**
- [Contributing](CONTRIBUTING.md)
- [Architecture Specification](docs/architecture/ARCHITECTURE.md)
- [Canonical Data Model](docs/model/CANONICAL_MODEL.md)
- [Universal Code Graph](docs/graph/UNIVERSAL_CODE_GRAPH.md)
- [Plugin System](docs/plugins/PLUGIN_SYSTEM.md)
- [Parser Framework (Tree-sitter)](docs/parser/PARSER_FRAMEWORK.md)
- [Software Knowledge Graph](docs/knowledge/SOFTWARE_KNOWLEDGE_GRAPH.md)
- [Requirements Graph](docs/requirements/REQUIREMENTS_GRAPH.md)
- [Verification Engine](docs/verification/VERIFICATION_ENGINE.md)
- [ADR Index](docs/adr/README.md)
- Per-crate README: `crates/<name>/README.md`

## Design Rules

0. **Follow [Engineering Standards](docs/engineering/ENGINEERING_STANDARDS.md)** — all implementation must comply.
1. **Traits and contracts first** — extend via documented public APIs; v1 porting pipeline is implemented in `s4-cli` + capability crates.
2. **No LLM provider dependencies** — `s4-llm` defines interfaces; providers are plugins.
3. **All cross-boundary I/O is artifact-ID based** — via `s4-storage`.
4. **LLM outputs are always `Proposed` lifecycle** — see `s4-knowledge`.
5. **Heuristic correspondences require manual confirmation** — never auto-certified as ported.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
