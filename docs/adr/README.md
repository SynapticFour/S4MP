# Architecture Decision Records

Formal ADRs referenced from [ARCHITECTURE.md](../architecture/ARCHITECTURE.md) will be recorded here as decisions are finalized.

**Template:** [0000-template.md](./0000-template.md)
**Process:** [Engineering Standards §11](../engineering/ENGINEERING_STANDARDS.md#11-architecture-decision-records-adrs)
**Cross-cutting proposals:** [RFC process](../rfc/README.md)

## Index

| ADR | Title | Status |
|-----|-------|--------|
| [ADR-001](./0001-knowledge-model-is-the-product.md) | Knowledge model is the product | Accepted |
| [ADR-002](./0002-content-addressed-artifact-store.md) | Content-addressed artifact store | Accepted |
| [ADR-003](./0003-usir-as-universal-interchange.md) | USIR as universal interchange | Accepted |
| [ADR-004](./0004-plugins-at-volatile-boundaries.md) | Plugins at volatile boundaries | Accepted |
| [ADR-005](./0005-llm-outputs-always-proposed.md) | LLM outputs always proposed | Accepted |
| [ADR-006](./0006-layered-graph-projections.md) | Layered graph projections | Accepted |
| [ADR-007](./0007-declarative-pipelines.md) | Declarative pipelines | Accepted |
| [ADR-008](./0008-rust-for-core-platform.md) | Rust for core platform | Accepted |
| [ADR-009](./0009-schema-first-apis.md) | Schema-first APIs | Accepted |
| [ADR-010](./0010-query-engine-independent-of-ai.md) | Query engine independent of AI | Accepted |
| [ADR-011](./0011-no-auto-apply-refactors-in-core.md) | No auto-apply refactors in core | Accepted |
| [ADR-012](./0012-blake3-for-artifact-ids.md) | Blake3 for artifact IDs | Accepted |
| [ADR-013](./0013-llvm-infrastructure-not-sonarqube.md) | LLVM infrastructure, not SonarQube rule engine | Accepted |
| [ADR-014](./0014-json-artifact-encoding-v0.1.md) | JSON artifact encoding for v0.1 | Accepted |
| [ADR-015](./0015-in-process-plugins-through-phase-5.md) | In-process plugins through Phase 5 | Accepted |
| [ADR-016](./0016-phase6-in-process-host-wasm-deferred.md) | Phase 6 in-process host; WASM still deferred | Accepted |

## Open

- Graph storage at scale
- Query language: S4QL vs Datalog/Cypher subset
- WASM plugin sandbox (deferred past Phase 6; see ADR-016)
