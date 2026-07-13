# SynapticFour Method Platform (S4MP)

A production-grade, modular, plugin-driven platform for software knowledge extraction, analysis, and certification.

**The knowledge model is the product. AI is one consumer among many.**

## Workspace

This repository is a [Cargo workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html) of 18 focused crates. Each crate owns a single concern, exposes a documented public API of traits and types, and compiles independently.

```
crates/
  s4-core            Foundation: IDs, errors, versioning
  s4-storage         Content-addressed artifact store contracts
  s4-events          Event bus contracts
  s4-plugin          Plugin system contracts
  s4-project         Project workspace contracts
  s4-parser          Universal parsing & USIR contracts
  s4-graph           Universal code graph contracts
  s4-knowledge       Software knowledge graph contracts
  s4-requirements    Requirements graph & traceability
  s4-metrics         Complexity & metrics contracts
  s4-analysis        Architecture & feature analysis
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

## Quick Start

```bash
# Build entire workspace
cargo build --workspace

# Check a single crate in isolation
cargo check -p s4-core
cargo check -p s4-knowledge

# Run CLI skeleton
cargo run -p s4-cli -- init .
cargo run -p s4-cli -- analyze
cargo run -p s4-cli -- query --expr all
```

## Documentation

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

1. **No business logic in this skeleton** — traits, types, and module boundaries only.
2. **No LLM provider dependencies** — `s4-llm` defines interfaces; providers are plugins.
3. **All cross-boundary I/O is artifact-ID based** — via `s4-storage`.
4. **LLM outputs are always `Proposed` lifecycle** — see `s4-knowledge`.

## License

MIT OR Apache-2.0
