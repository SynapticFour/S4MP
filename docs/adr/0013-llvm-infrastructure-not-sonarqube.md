# ADR-013: Platform as LLVM infrastructure, not SonarQube rule engine

- **Status:** Accepted
- **Date:** 2025-07-13
- **Deciders:** SynapticFour architecture baseline
- **Related:** [ARCHITECTURE.md](../architecture/ARCHITECTURE.md), [ADR-003](./README.md#index) (USIR), [ADR-004](./README.md#index) (Plugins), [Parser Framework](../parser/PARSER_FRAMEWORK.md)

## Context

Static analysis platforms often evolve into **collections of rules applied directly to source text** — the SonarQube model. That approach works for linting but fails as a long-lived knowledge platform:

- A new language requires reimplementing or reconfiguring rules across the stack.
- Analysis, planning, verification, and certification become tightly coupled to rule versions.
- LLM providers, parsers, and verification techniques churn every few years; a rule-centric core must be rewritten repeatedly.

S4MP's goal is a **5+ year stable platform** where parsers, metrics, LLM models, and verification methods are replaceable without redesigning the center.

The LLVM project solved an analogous problem: many frontends (Clang, Rust, Swift) lower to a **common intermediate representation (IR)**, on which optimization and backend passes compose independently.

## Decision

S4MP is an **infrastructure platform**, not a rule engine:

1. **USIR is the stable core** — analogous to LLVM IR. Language frontends (Tree-sitter parser plugins) lower source into USIR; everything downstream consumes USIR or artifacts derived from it.

2. **Graphs are projections**, not alternate sources of truth:
   - **Universal Code Graph (UCG)** — structure and behavior from USIR + linker
   - **Software Knowledge Graph (SKG)** — meaning curated and inferred from USIR, docs, and human input
   - **Requirements Graph** — contractual intent from SOW/OpenAPI (parallel input, not lowered from USIR)

3. **Capabilities are composable passes** — analysis, planning, transformation, verification, and certification are **pass plugins** that read and write CAS artifacts. They do not embed language-specific or vendor-specific logic in core crates.

4. **The core never ships a fixed rule catalog.** Rules live in verifier/analyzer plugins. Core defines pass contracts, artifact schemas, and orchestration — not business rules.

5. **Pass pipeline order** (canonical):

   ```
   Repository → Language Frontends → USIR
     → Graph materialization (UCG, SKG)
     → Analysis Passes → Planning Passes → Transformation Passes
     → Verification Passes → Certification
   ```

   Requirements ingestion runs in parallel to parsing; trace edges connect Requirements to UCG/SKG at verification time.

## Consequences

### Positive

- New languages = new frontend plugin; analyzers unchanged.
- New LLM or verification technique = new pass plugin; USIR and graphs unchanged.
- Passes compose and invalidate incrementally via artifact DAG (ADR-002).
- Clear anti-pattern guard: reject designs that hard-code rule collections in `s4-core` or `s4-analysis`.

### Negative

- Higher upfront investment in USIR schema stability and pass orchestration vs. shipping quick lint rules.
- Requirements Graph requires explicit cross-graph trace machinery (not automatic from USIR).
- Pass manager implementation deferred — traits exist; orchestrator is future work.

### Neutral

- Individual passes may still implement "rules" internally (e.g. security linter) — the distinction is **where** they live (plugins), not whether rules exist.
- SonarQube-like **findings** remain valid outputs; they are artifacts emitted by analysis passes, not the platform definition.

## Alternatives Considered

| Alternative | Why rejected |
|-------------|--------------|
| **SonarQube-style rule monolith in core** | Every language and vendor change rewrites core; contradicts plugin model (ADR-004). |
| **Single unified mega-graph (no USIR)** | Loses language-agnostic interchange; parsers and analyzers re-couple. |
| **Requirements lowered from USIR only** | Ignores contractual SOW intent; legally meaningful requirements are not code-derived. |
| **No pass abstraction — ad hoc pipelines** | Prevents composability, incremental invalidation, and certification replay. |

## Compliance

| Check | Mechanism |
|-------|-----------|
| No rule catalog in Tier 0–2 crates | Code review + `deny.toml` bans on analyzer SDKs in core |
| USIR remains interchange layer | ADR-003; parser plugins must emit USIR artifacts |
| New capabilities are plugins | `s4-plugin` trait roles; PR checklist |
| Pipeline documented | [ARCHITECTURE.md §9](../architecture/ARCHITECTURE.md#9-processing-pipeline-llvm-model) |
| Pass orchestrator | Future RFC; until then, documented pass ordering is normative |
