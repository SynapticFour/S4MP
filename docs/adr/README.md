# Architecture Decision Records

Formal ADRs referenced from [ARCHITECTURE.md](../architecture/ARCHITECTURE.md) will be recorded here as decisions are finalized.

**Template:** [0000-template.md](./0000-template.md)
**Process:** [Engineering Standards §11](../engineering/ENGINEERING_STANDARDS.md#11-architecture-decision-records-adrs)
**Cross-cutting proposals:** [RFC process](../rfc/README.md)

## Index

| ADR | Title | Status |
|-----|-------|--------|
| ADR-001 | Knowledge model is the product | Accepted |
| ADR-002 | Content-addressed artifact store | Accepted |
| ADR-003 | USIR as universal interchange | Accepted |
| ADR-004 | Plugins at volatile boundaries | Accepted |
| ADR-005 | LLM outputs always proposed | Accepted |
| ADR-008 | Rust for core platform | Accepted |
| ADR-012 | Blake3 for artifact IDs | Accepted |
| ADR-013 | LLVM infrastructure, not SonarQube rule engine | Accepted |
| [ADR-014](./0014-json-artifact-encoding-v0.1.md) | JSON artifact encoding for v0.1 | Accepted |
| [ADR-015](./0015-in-process-plugins-through-phase-5.md) | In-process plugins through Phase 5 | Accepted |

## Open

- Graph storage at scale
- Query language: S4QL vs Datalog/Cypher subset
