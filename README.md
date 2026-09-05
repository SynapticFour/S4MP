# S4MP — heuristic Java↔Rust port maps

Freeze status (2026-09): [STATUS.md](STATUS.md).


> **Development paused (2026-09).** Internal Java↔Rust heuristic port-map tooling.
> Not a Synaptic Four product SKU. Not Ferrum. `s4 certify` is not semantic
> equivalence and not certification of a port.

A **local CLI** that parses Java and Rust with Tree-sitter, builds a name graph, and suggests correspondences by **token Jaccard** (optional signatures). Maturity: `heuristic-map-v2`.

**This is not** a production knowledge platform, not semantic equivalence, not a loadable plugin runtime, and not certification of a port. `s4 certify` evaluates policy over verification counters; the default policy **rejects** heuristic-only maps (zero manually confirmed `Ported` rows).

Roadmap leftovers (HTTP API, UI, planner, WASM sandbox, networked LLM) are **parked stubs or specs**, not shipped software.

**Maintainer:** one person (`Synaptic Four`). Treat availability as best-effort. There is no review roster.

## Quick Start (intended loop)

The product is a **reviewable port map**, not a certificate of semantic equivalence.

```
init → source add → graph build → map suggest → map show → map confirm → diff → verify → certify
```

`s4 certify` is Valid only after **at least one** row is manually `Ported`. Heuristic suggestions stay `Diverged`.

### Smoke test (no network)

```bash
cargo build --workspace
make e2e-fixture
```

That runs the bundled `tests/fixtures/mini-port/` trees through suggest, review, confirm, and certify.

### Your own trees

```bash
cargo run -p s4-cli -- init .
cargo run -p s4-cli -- source add my-java --local /path/to/java --lang java
cargo run -p s4-cli -- source add my-rust --local /path/to/rust --lang rust
cargo run -p s4-cli -- graph build --source my-java
cargo run -p s4-cli -- graph build --source my-rust
cargo run -p s4-cli -- map suggest --java my-java --rust my-rust
cargo run -p s4-cli -- map show --java my-java --rust my-rust --status diverged
cargo run -p s4-cli -- map confirm --name add --java my-java --rust my-rust
cargo run -p s4-cli -- diff --java my-java --rust my-rust
cargo run -p s4-cli -- verify --java my-java --rust my-rust
cargo run -p s4-cli -- certify --java my-java --rust my-rust
```

### Optional: GATK HaplotypeCaller slice

From the repository root, with your Rust port at `../my-hc-port`:

```bash
make sources RUST_LOCAL=../my-hc-port
make show          # graph → map suggest → table with short ids
make diff          # Markdown report with `id=` on every row
```

Then confirm from the table or report:

```bash
cargo run -p s4-cli -- map confirm --id <12-char-prefix> --java gatk-java-hc --rust hc-rust
# or
cargo run -p s4-cli -- map confirm --name <symbol> --java gatk-java-hc --rust hc-rust
```

See [`Makefile`](Makefile) for targets (`sources`, `graph`, `map`, `show`, `diff`, `verify`, `certify`, `e2e-fixture`, `install-hooks`, `clean-cache`).

**Install on PATH (optional):** there is no crates.io release and no tagged `0.1.0`. From this checkout:

```bash
cargo install --path crates/s4-cli --locked
```

**Full beginner guide:** [Porting Workflow](docs/guides/PORTING_WORKFLOW.md)

**CLI command reference:** [`crates/s4-cli/README.md`](crates/s4-cli/README.md)

**Docs index:** [`docs/README.md`](docs/README.md)

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

This repository is a Cargo workspace of **15 built crates** plus **3 parked** trait stubs (`s4-planner`, `s4-api`, `s4-ui`) that are excluded from the default build. The live surface is `s4-cli`.

```
crates/
  s4-core            Foundation: IDs, errors, versioning
  s4-storage         Content-addressed artifact store (pointers for indexes)
  s4-events          In-process event recorder
  s4-plugin          Manifest types + in-process registry (not a plugin loader)
  s4-project         Source ingest (git URL allowlist) & snapshots (size caps)
  s4-parser          Tree-sitter Java/Rust frontends → USIR
  s4-graph           In-memory code graph + filter query
  s4-knowledge       Naming-heuristic concepts (Proposed)
  s4-requirements    Requirements JSON + name traces
  s4-metrics         Basic graph counts
  s4-analysis        Lowering, exclusive correspondence, diff reports
  s4-verification    Coverage/threshold runs (not semantic equivalence)
  s4-certification   Policy over VerificationRun (`min_ported:1` default)
  s4-llm             Offline heuristic reasoner (Proposed only)
  s4-cli             `s4` CLI
```

Parked (not built): `s4-planner`, `s4-api`, `s4-ui`.

## Dependency Tiers

Dependency direction is strictly **inward**:

```
Surfaces (s4-cli)
    ↓
Quality (s4-verification, s4-certification)
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

Architecture documents under `docs/` describe a **target** model. They are not a feature checklist.

### Guides

- **[Implementation Roadmap](docs/guides/IMPLEMENTATION_ROADMAP.md)** — shipped vs parked
- **[Porting Workflow](docs/guides/PORTING_WORKFLOW.md)** — review → confirm → certify
- **[Docs index](docs/README.md)** — what is product vs target spec

### Architecture & Standards

- [Engineering Standards](docs/engineering/ENGINEERING_STANDARDS.md)
- [Contributing](CONTRIBUTING.md)
- [Architecture Specification](docs/architecture/ARCHITECTURE.md) (target model)
- [ADR Index](docs/adr/README.md)
- Per-crate README: `crates/<name>/README.md`

## Design Rules

0. **Follow [Engineering Standards](docs/engineering/ENGINEERING_STANDARDS.md)** when implementing.
1. **Do not market specs as features.** Architecture docs under `docs/` are a target model; the CLI is the product.
2. **No LLM provider dependencies** — `s4-llm` is heuristic/offline unless a future provider is added as an explicit crate.
3. **Primary knowledge artifacts are CAS** — envelopes live under their Blake3 hash; secondary indexes are pointers.
4. **LLM/heuristic outputs are always `Proposed`.**
5. **Heuristic correspondences are never `Ported`.** Manual confirmation only. Default `s4 certify` fails until that exists.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option. See [LICENSE](LICENSE).
