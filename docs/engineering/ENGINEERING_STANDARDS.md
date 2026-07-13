# Engineering Standards
## S4MP Platform — Mandatory Baseline v0.1

> **Status:** Effective immediately — all new implementation must comply  
> **Authority:** This document supersedes ad-hoc conventions. Deviations require an ADR or RFC.  
> **Related:** [Architecture](../architecture/ARCHITECTURE.md), [ADR Index](../adr/README.md), [RFC Process](../rfc/README.md)

---

## 1. Scope and Authority

These standards apply to **every crate, plugin, tool, and CI job** in the S4MP repository.

| Rule | Enforcement |
|------|-------------|
| Standards are mandatory before feature work | PR checklist + CI gates |
| Exceptions require written approval | ADR (architectural) or RFC (cross-cutting change) |
| Config wins over memory | `rustfmt.toml`, `deny.toml`, workspace lints, CI |
| When standards and architecture conflict | Architecture spec wins; update standards via RFC |

**Principle:** Prefer **automated enforcement** over documentation that nobody reads.

---

## 2. Coding Style

### 2.1 Formatter — `rustfmt`

All Rust code is formatted with **stable `rustfmt`** using the repository [`rustfmt.toml`](../../rustfmt.toml).

```bash
cargo fmt --all
```

- PRs must be rustfmt-clean. CI fails on diff.
- No `#![rustfmt::skip]` except in generated code (must comment why).
- Edition: **2021** (workspace default).

### 2.2 Linter — `clippy`

Workspace lints in root [`Cargo.toml`](../../Cargo.toml):

| Lint | Level |
|------|-------|
| `missing_docs` | warn (public items) |
| `unsafe_code` | **forbid** (workspace default) |
| `clippy::all` | warn |
| `clippy::pedantic` | warn |

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

- Allowed `#[allow(clippy::…)]` only with comment citing issue or ADR.
- `#![allow(…)]` at crate level is forbidden except in test/bench crates with ADR.

### 2.3 General Rust Style

| Topic | Standard |
|-------|----------|
| Line length | 100 columns (rustfmt) |
| Imports | `rustfmt` order; group `std`, external, `s4_*`, `crate` |
| `pub use` | Re-export public API from `lib.rs`; hide internals |
| Derives | Prefer `Debug, Clone` on value types; `Eq, Hash` when used as keys |
| Serialization | `serde` with explicit `rename_all = "snake_case"` on enums |
| Async | `async-trait` for trait objects; no async in `s4-core` |
| Panics | Forbidden in library `s4-*` crates (see §5) |
| `unwrap` / `expect` | Tests and benchmarks only |
| Magic numbers | Named constants with doc comment |
| Feature flags | `Cargo.toml` `[features]`; default features minimal |

### 2.4 Type Design

- **Newtype** wrappers for IDs (`ArtifactId`, not raw `String`).
- Prefer **`&str`** over `String` in read-only APIs.
- Prefer **`Cow<'_, str>`** when ownership is ambiguous at API boundary.
- Use **`NonZero*`** types when zero is invalid.
- Avoid **`RefCell`** in concurrent code; use message passing or `Arc` + explicit sync.

---

## 3. Module Layout

### 3.1 Crate Structure

Every `s4-*` crate follows this layout:

```
crates/s4-<name>/
├── Cargo.toml
├── README.md              # Responsibility, public API table, tier, dependencies
├── benches/                 # Optional; criterion benchmarks
│   └── <name>.rs
├── tests/                   # Integration tests
│   └── <scenario>.rs
└── src/
    ├── lib.rs               # Crate docs, module declarations, pub re-exports
    ├── <domain>.rs          # One file per cohesive domain module
    └── <domain>/
        └── <sub>.rs         # Submodules when file exceeds ~400 lines
```

### 3.2 `lib.rs` Pattern

```rust
//! # s4-<name>
//!
//! One-line purpose. Link to architecture doc if applicable.
//!
//! Tier: N — <tier name>

#![warn(missing_docs)]

/// Domain module — one-line description.
pub mod domain;

pub use domain::{PublicType, PublicTrait};
```

Rules:

- **One concern per module file** — align with architecture boundaries.
- **`mod` tests** inline only for unit tests tightly coupled to one type; prefer `tests/` for integration.
- **No `main.rs`** in contract crates — binaries live in `s4-cli` or future `*-engine` impl crates.
- **Impl crates** (future): `s4-storage-engine`, `s4-graph-engine` — may have `src/bin/` with ADR.

### 3.3 Dependency Tier Enforcement

Crates must not depend upward (see README tier diagram). CI checks via `cargo deny` + custom tier script (future).

| Tier | Crates |
|------|--------|
| 0 | `s4-core` |
| 1 | `s4-storage`, `s4-events`, `s4-plugin`, `s4-project` |
| 2 | `s4-parser`, `s4-graph`, `s4-knowledge`, `s4-requirements`, `s4-metrics`, `s4-analysis` |
| 3 | `s4-planner`, `s4-verification`, `s4-certification`, `s4-llm` |
| 4 | `s4-api`, `s4-cli`, `s4-ui` |

---

## 4. Naming Conventions

### 4.1 Crates and Packages

| Kind | Pattern | Example |
|------|---------|---------|
| Contract crate | `s4-<domain>` | `s4-knowledge` |
| Implementation crate | `s4-<domain>-engine` | `s4-storage-engine` (future) |
| Plugin crate | `s4-plugin-<name>` | `s4-plugin-rust-ts` (future) |
| Binary | same as surface crate | `s4-cli` → binary `s4` |

### 4.2 Types and Traits

| Kind | Convention | Example |
|------|------------|---------|
| ID newtypes | `<Entity>Id` | `ArtifactId`, `RequirementId` |
| Traits (behavior) | Verb or role noun | `ParsePipeline`, `Verifier`, `ArtifactStore` |
| Error enums | `<Crate>Error` or `S4Error` (core only) | `S4Error` |
| Result alias | `Result<T>` | `s4_core::Result<T>` |
| Snapshot types | `<Domain>Snapshot` | `UcgSnapshot`, `RequirementsSnapshot` |
| Artifact payloads | `<Kind>Artifact` or domain noun | `VerificationRun` |
| Builder | `<Type>Builder` | `InvariantSetBuilder` (future) |

### 4.3 Functions and Methods

| Kind | Convention | Example |
|------|------------|---------|
| Constructor | `new`, `with_*`, `from_*` | `ArtifactId::from_hex` |
| Conversion | `into_*`, `as_*`, `to_*` | `to_bytes()` |
| Predicate | `is_*`, `has_*` | `is_proposed()` |
| Fallible parse | `try_from`, `parse` | `parse_document()` |
| Async | same names, no `async_` prefix | `store.put(...).await` |

### 4.4 Files and Docs

| Kind | Convention |
|------|------------|
| Rust modules | `snake_case.rs` |
| Architecture specs | `SCREAMING_SNAKE.md` in `docs/<area>/` |
| ADRs | `docs/adr/NNNN-short-title.md` |
| RFCs | `docs/rfc/NNNN-short-title.md` |
| Schemas | `schemas/<name>.schema.json` |

### 4.5 Constants and Config

- `SCREAMING_SNAKE_CASE` for constants
- Environment variables: `S4MP_<AREA>_<NAME>` (future runtime)

---

## 5. Error Handling

### 5.1 Platform Error Type

- **`s4-core::S4Error`** is the root error for cross-crate boundaries.
- Crate-local errors use **`thiserror`** and convert via **`From`** or explicit mapping to `S4Error`.
- **`anyhow`** is forbidden in library crates; allowed in `s4-cli` binary only.

```rust
use s4_core::{Result, S4Error};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),
}

impl From<ParseError> for S4Error {
    fn from(e: ParseError) -> Self {
        S4Error::Other(e.to_string())
    }
}
```

### 5.2 Rules

| Rule | Rationale |
|------|-----------|
| **No panic** in library code paths | Artifacts and graphs must fail gracefully |
| **`Result` over Option`** for operations that can fail | Explicit error propagation |
| **Structured variants** over stringly `Other` | Add variants before abusing `Other` |
| **Context at boundaries** | Plugin/storage errors include IDs |
| **No error swallowing** | Log + propagate; never empty `catch { }` |
| **Schema errors** | Use `SchemaVersionMismatch` variant |

### 5.3 Plugin Errors

Plugins return structured errors through the plugin host; host maps to `S4Error::Plugin { plugin_id, message }`. Plugin-internal details go in artifact diagnostics, not logs alone.

---

## 6. Logging

### 6.1 Crate: `tracing`

All runtime logging uses **`tracing`**. **`log`** crate is not used directly.

| Level | Use |
|-------|-----|
| `ERROR` | Unrecoverable failure; operation aborted |
| `WARN` | Degraded behavior; stale data; retry succeeded |
| `INFO` | Lifecycle: snapshot created, verification started/completed |
| `DEBUG` | Developer detail: shard loaded, cache hit |
| `TRACE` | Hot-path internals (off in production default) |

### 6.2 Structured Fields

```rust
tracing::info!(
    snapshot_id = %snapshot_id,
    artifact_count = manifest.len(),
    "requirements snapshot materialized"
);
```

- Use **`%`** for Display, **`?`** for Debug on IDs.
- **Never log secrets** — tokens, credentials, raw SOW with PII (see §19).
- **Never log full artifact payloads** at INFO+ — log `ArtifactId` only.

### 6.3 Log Initialization

- **`s4-cli`** and future servers initialize `tracing-subscriber`.
- Library crates **do not** init subscribers — inject via caller.

---

## 7. Tracing (Distributed)

### 7.1 Spans

Operations that cross crate or I/O boundaries emit spans:

| Operation | Span name | Fields |
|-----------|-----------|--------|
| Parse file | `s4.parse` | `project_id`, `path`, `language` |
| Store get/put | `s4.storage.{get,put}` | `artifact_id` |
| Verification run | `s4.verify.compare` | `comparison_id`, `baseline`, `candidate` |
| Plugin invoke | `s4.plugin.invoke` | `plugin_id`, `method` |

### 7.2 Conventions

- Parent span passed via `tracing` context; no global span stack hacks.
- **`#[instrument]`** on public async/sync entry points in impl crates.
- **`skip`** large payloads in instrument args.
- OpenTelemetry export: future ADR; design for `tracing-opentelemetry` compatibility now.

### 7.3 Metrics vs Traces

- **Traces:** request-scoped latency and causality.
- **Metrics:** counters/histograms in `s4-metrics` (future impl) — not duplicated in log lines.

---

## 8. Testing Strategy

### 8.1 Test Pyramid

| Layer | Location | Purpose | Required |
|-------|----------|---------|----------|
| **Unit** | `src/**/*.rs` `#[cfg(test)]` | Pure logic, type invariants | Every public function with logic |
| **Integration** | `tests/*.rs` | Crate API, serialization roundtrips | Every crate |
| **Contract** | `tests/contract/` or workspace test crate | Cross-crate trait conformance | Plugin interfaces, artifact schemas |
| **Snapshot** | `insta` (future) | Stable JSON/CBOR artifact output | Graph manifests, USIR |
| **End-to-end** | `tests/e2e/` (future) | CLI pipelines on fixture repos | Release gate |

### 8.2 Conventions

```bash
cargo test --workspace
cargo test -p s4-core
```

- Test modules: `#[cfg(test)] mod tests { use super::*; … }`
- Test names: `snake_case` describing behavior — `rejects_invalid_artifact_id`
- Fixtures: `tests/fixtures/` or `crates/<crate>/tests/data/`
- **Deterministic:** no network in unit/integration tests; mock via traits.
- **No flaky tests** in CI — quarantine with issue link or fix immediately.

### 8.3 Coverage Expectations

| Tier | Line coverage target |
|------|---------------------|
| `s4-core`, `s4-storage` | ≥ 85% when impl lands |
| Domain contract crates | ≥ 70% (types + serialization) |
| `s4-cli` | Critical paths covered |

Coverage is informational in CI v0.1; becomes gate when impl crates exist.

### 8.4 Property and Fuzz Testing

- **Property tests (`proptest`)** for parsers, ID parsing, artifact roundtrips.
- **Fuzz targets (`cargo-fuzz`)** for parser plugins and artifact decoders — mandatory before 1.0 for untrusted input paths.

---

## 9. Benchmarks

### 9.1 Tooling

- **`criterion`** for microbenchmarks in `benches/`.
- **`cargo bench -p <crate>`** — not run in default CI (too slow); weekly job + PR label `bench`.

### 9.2 Requirements

| Area | Benchmark | Budget reference |
|------|-----------|------------------|
| Blake3 hash (1 MiB) | `benches/hash.rs` | §20 |
| Artifact serialize/deserialize | per crate | §20 |
| Graph diff (10k nodes) | `s4-graph-engine` (future) | §20 |
| USIR parse (1k LOC fixture) | parser plugins | §20 |

### 9.3 Benchmark Hygiene

- Check in **baseline** via criterion (optional); report regression > 10% in PR.
- Use `black_box` for inputs; warm up included.
- Separate `--release` only; never bench debug builds in reports.

---

## 10. Documentation Style

### 10.1 Rustdoc

Every **public** item has `///` doc comment.

```rust
/// Content-addressed identifier for immutable artifacts.
///
/// Derived from Blake3 hash of canonical serialized bytes.
#[derive(...)]
pub struct ArtifactId([u8; 32]);
```

- First line: **one sentence summary**.
- Include **`# Examples`** for non-trivial APIs when impl exists.
- Link to architecture docs with `[Architecture](…)` in crate-level docs.
- **`#![warn(missing_docs)]`** on every crate; workspace lint escalates to deny in CI.

### 10.2 Crate README

Every crate has `README.md`:

| Section | Content |
|---------|---------|
| Responsibility | One paragraph |
| Public API | Module table |
| Dependencies | Internal tier deps |
| Tier | Number and name |

### 10.3 Architecture Documentation

- Location: `docs/<domain>/<NAME>.md`
- Status line: `Design baseline`, `Accepted`, `Deprecated`
- Cross-link related specs; no duplicate normative content — link instead.

### 10.4 Comments in Code

- **Why, not what** — business rules, non-obvious invariants, safety preconditions.
- **No commented-out code** in main branch.
- **TODO** format: `// TODO(#issue): description` — bare TODO fails clippy lint (future).

---

## 11. Architecture Decision Records (ADRs)

### 11.1 When to Write an ADR

- Choosing between architectural alternatives (storage engine, WASM vs in-process)
- Changing dependency tier rules
- Allowing `unsafe`, new license, or MSRV bump
- Anything that is **hard to reverse**

### 11.2 Format

- Location: `docs/adr/NNNN-short-title.md`
- Template: [`docs/adr/0000-template.md`](../adr/0000-template.md)
- Status: `Proposed` → `Accepted` | `Rejected` | `Superseded by ADR-NNNN`

### 11.3 Process

1. Author opens PR with ADR in `Proposed` state.
2. Review by at least one maintainer; discussion in PR.
3. On merge, status → `Accepted`; index updated in [`docs/adr/README.md`](../adr/README.md).
4. Superseded ADRs remain for history; never delete.

### 11.4 Relationship to Architecture Specs

- **Architecture specs** (`docs/architecture/`, `docs/graph/`, etc.) describe the **system**.
- **ADRs** record **decisions** and alternatives rejected.

---

## 12. RFC Process

RFCs are for **cross-cutting changes** affecting multiple crates or external contributors — larger than a single ADR, smaller than a rewrite.

### 12.1 When to Use RFC vs ADR

| RFC | ADR |
|-----|-----|
| New query language syntax | Blake3 vs SHA256 for IDs |
| Public plugin SDK breaking change | In-process plugins phase 1 |
| CI policy change affecting all crates | Graph storage backend choice |
| Versioning policy change | |

### 12.2 Process

See [`docs/rfc/README.md`](../rfc/README.md).

1. Copy [`docs/rfc/0000-template.md`](../rfc/0000-template.md) → `docs/rfc/NNNN-title.md`
2. Status `Draft` → PR for review (minimum 5 business days comment period for breaking changes)
3. `Accepted` → implementation tracking issue; link ADRs spawned from RFC
4. `Rejected` / `Withdrawn` — kept for history

---

## 13. Versioning

### 13.1 Crate Semver (Rust)

- Workspace version: **`0.y.z`** until 1.0.
- **0.y.z:** `y` = breaking API change to public traits/types; `z` = compatible fix/feature.
- After 1.0: strict [semver](https://semver.org/) on all public API.

Breaking changes to public traits require:
- ADR or RFC
- CHANGELOG entry (when CHANGELOG introduced)
- Deprecation attribute one minor release before removal (post-1.0)

### 13.2 Schema Versioning

Artifact and graph payloads carry **`SchemaVersion`** (`s4-core`):

```
{ major, minor, patch }
```

| Change | Bump |
|--------|------|
| Breaking field removal/rename | major |
| Additive optional fields | minor |
| Documentation-only schema | patch |

Consumers must reject unsupported **major** versions with `SchemaVersionMismatch`.

### 13.3 Plugin API Version

`ApiVersion` independent from crate semver; documented in plugin system spec. Plugins declare compatible range; host rejects mismatch.

### 13.4 Documentation Versioning

Architecture specs carry `v0.N` in title. Increment on material semantic change; link from ADR when decision-driven.

---

## 14. Git Workflow

### 14.1 Branching

| Branch | Purpose |
|--------|---------|
| `main` | Always deployable; protected |
| `feature/<issue>-<short>` | New work |
| `fix/<issue>-<short>` | Bug fixes |
| `docs/<topic>` | Documentation-only |
| `rfc/<number>-<title>` | RFC drafts |

No long-lived development branches. Rebase preferred over merge commits on feature branches.

### 14.2 Commits

- **Conventional intent:** imperative subject, ≤ 72 chars
- Body explains **why**, not only what
- One logical change per commit when possible
- Reference issue: `Fixes #123` in body

```
Add Verification Engine architecture specification.

Define version comparison with confidence-scored verdicts and explicit gaps.
```

### 14.3 Pull Requests

- **Required:** fmt, clippy, test pass; docs for public API changes
- **Required:** link issue or ADR/RFC
- **Squash merge** to `main` default (linear history)
- **No force-push** to `main`
- Draft PR for WIP; mark ready when CI green

### 14.4 Review

- At least **one approval** for code; **two** for security-sensitive or RFC/ADR
- Author merges after approval (when permissions allow)

---

## 15. Dependency Policy

### 15.1 Workspace Dependencies

- **All external deps** declared in root `[workspace.dependencies]`.
- Crates reference via `{ workspace = true }` — no duplicate version pins.

### 15.2 `cargo-deny`

[`deny.toml`](../../deny.toml) enforced in CI:

| Check | Policy |
|-------|--------|
| Licenses | MIT, Apache-2.0, BSD-*, ISC, Unicode-3.0 |
| Wildcards | **deny** |
| Multiple versions | warn; justify or consolidate |
| Unknown registry | deny |
| Banned crates | LLM/HTTP in core (see deny.toml) |

### 15.3 Adding Dependencies

1. Add to `[workspace.dependencies]` with pinned version
2. Run `cargo deny check`
3. PR description: **why needed**, **license**, **tier appropriate**
4. Prefer **std + small deps**; avoid heavy frameworks in contract crates

### 15.4 Internal Dependencies

- Contract crates (`s4-*`) minimize external deps — `serde`, `thiserror`, `tracing` acceptable.
- **No dependency from inner to outer tier.**

### 15.5 Vulnerability Response

- `cargo deny advisories` in CI — fail on **unsound** and **high** severity for direct deps.
- Patch within 14 days or document waiver ADR.

---

## 16. Unsafe Rust Policy

### 16.1 Default: Forbidden

Workspace lint: **`unsafe_code = forbid`** on all contract crates.

### 16.2 When Allowed

`unsafe` requires **all** of:

1. **Accepted ADR** with safety invariants documented
2. **`// SAFETY:`** comment on every block explaining preconditions
3. **`unsafe` isolated** in dedicated module or `-engine` crate
4. **Review** by second maintainer with unsafe experience
5. **Tests** covering safety boundaries; miri where applicable

### 16.3 Preferred Alternatives

- Safe abstractions (`bytes`, `bytemuck` with ADR)
- FFI behind `*-sys` crate with ADR
- WASM sandbox for untrusted plugins (plugin system spec)

---

## 17. MSRV Policy

### 17.1 Current MSRV

**Rust 1.75** — declared in `[workspace.package] rust-version`.

[`rust-toolchain.toml`](../../rust-toolchain.toml) pins **stable** with `rustfmt` and `clippy`.

### 17.2 Rules

| Action | Requirement |
|--------|-------------|
| MSRV bump | ADR + update `rust-version`, CI, README |
| New language features | Must work on MSRV |
| `edition` bump | RFC + coordinated workspace change |

CI runs `cargo +1.75 check --workspace` when MSRV lags stable (MSRV job).

---

## 18. CI/CD Strategy

### 18.1 Pipeline (GitHub Actions)

See [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml).

| Job | Trigger | Gates |
|-----|---------|-------|
| **check** | every push/PR | `cargo check --workspace` |
| **fmt** | every push/PR | `cargo fmt --all --check` |
| **clippy** | every push/PR | `clippy -D warnings` |
| **test** | every push/PR | `cargo test --workspace` |
| **deny** | every push/PR | `cargo deny check` |
| **msrv** | PR + nightly schedule | MSRV toolchain |
| **doc** | PR | `cargo doc --no-deps` no warnings |
| **bench** | weekly / label | criterion, non-blocking initially |

### 18.2 Caching

- `Swatinem/rust-cache` for registry and target dir.

### 18.3 Release (Future)

- Tags `v0.y.z` trigger release workflow
- Crates.io publish for public crates (policy TBD)
- SBOM artifact generation

### 18.4 Failure Policy

- **Red main is emergency** — revert or fix within 24h
- No `--no-verify` on hooks (when hooks added)

---

## 19. Security Policy

### 19.1 Threat Model (Summary)

| Asset | Risk |
|-------|------|
| Artifact store | Tampering, unauthorized read |
| Plugin execution | Malicious WASM/native plugin |
| LLM prompts | Prompt injection, data exfiltration |
| SOW / requirements | PII in logs and artifacts |

### 19.2 Secure Development Rules

| Rule | Standard |
|------|----------|
| Secrets | Never in repo; `.env` gitignored; use env vars |
| Input validation | All external bytes (files, plugins, network) untrusted |
| Deserialization | `serde` with size limits; reject unknown major schema versions |
| Dependencies | deny.toml + advisory CI |
| Plugins | Trust tiers (plugin system spec); WASM sandbox for third-party |
| LLM | No auto-execution of LLM-suggested code; lifecycle `proposed` |
| Logging | No secrets, tokens, or raw PII at INFO+ |

### 19.3 Vulnerability Reporting

- Report to: **security@synapticfour.com** (placeholder — update when public)
- Response target: acknowledgment 48h, triage 7 days
- Embargo coordinated disclosure for confirmed issues

### 19.4 Security Review Triggers

- New `unsafe` block
- New network surface (`s4-api` impl)
- Plugin sandbox changes
- Cryptographic primitive change

---

## 20. Performance Budgets

Initial budgets for implementation phases. **Exceeding budget requires ADR** with profiling evidence.

### 20.1 Latency (single-threaded, release, representative hardware)

| Operation | Budget (p95) | Notes |
|-----------|--------------|-------|
| Artifact put (64 KiB) | ≤ 5 ms | includes Blake3 |
| Artifact get (64 KiB) | ≤ 2 ms | local CAS |
| USIR parse 1k LOC Rust | ≤ 200 ms | excl. tree-sitter load |
| UCG incremental diff 10k nodes | ≤ 500 ms | |
| Requirements trace query (1 req) | ≤ 50 ms | indexed |
| Version comparison orchestration overhead | ≤ 100 ms | excl. plugins/tests |

### 20.2 Memory

| Context | Budget |
|---------|--------|
| CLI idle | ≤ 50 MiB RSS |
| Parse single 10k LOC file | ≤ 200 MiB peak |
| Graph snapshot 100k nodes | ≤ 1 GiB resident (streaming preferred) |

### 20.3 Binary Size

| Target | Budget |
|--------|--------|
| `s4` CLI stripped | ≤ 25 MiB (phase 1); justify if larger |

### 20.4 CI Performance

| Job | Budget |
|-----|--------|
| Full CI pipeline | ≤ 15 min |
| `cargo check --workspace` | ≤ 3 min (cached) |

---

## 21. Pre-Implementation Checklist

Before merging **any feature implementation**, confirm:

- [ ] Aligns with architecture spec and dependency tier
- [ ] `cargo fmt`, `clippy -D warnings`, `test` pass
- [ ] Public API documented (`rustdoc` + crate README if new module)
- [ ] Errors use `Result` / `S4Error`; no new panics
- [ ] `tracing` spans on I/O boundaries (impl crates)
- [ ] Tests added for new behavior
- [ ] No new deps without deny check + justification
- [ ] No `unsafe` without ADR
- [ ] Performance-sensitive paths noted; benchmark if applicable
- [ ] ADR/RFC linked if architectural or cross-cutting

---

## 22. Document History

| Version | Date | Change |
|---------|------|--------|
| v0.1 | 2025-07-13 | Initial mandatory baseline |
