# SynapticFour Method Platform (S4MP)
## Architecture Specification v0.1

> **Status:** Approved skeleton baseline  
> **Principle:** Stable contracts at the center, volatile implementations at the edge.

---

## 1. Executive Summary

S4MP is a **knowledge platform for software systems**. It ingests repositories, produces a universal semantic representation, materializes multiple typed graphs, and exposes them to humans, analyzers, and AI reasoners through a single query and artifact model.

| Principle | Decision |
|-----------|----------|
| Product | The **knowledge model** (graphs + provenance + time), not any AI feature |
| AI role | One **consumer/producer** among many; never a dependency |
| Modularity | **Plugin-driven** at every volatile boundary |
| Language | **Language-agnostic core**; language specifics live in plugins |
| Storage | **Content-addressed immutable artifacts** (Git-like) |
| IR | **Universal Semantic IR (USIR)** as the stable interchange layer |
| Evolution | **Schema-versioned** everything; breaking changes are explicit migrations |

---

## 2. Architectural Layers

Dependency direction is strictly **inward** (outer layers depend on inner; never the reverse).

```
┌─────────────────────────────────────────────────────────────────┐
│  SURFACES: CLI · HTTP/gRPC API · SDK · IDE extensions           │
├─────────────────────────────────────────────────────────────────┤
│  ORCHESTRATION: Pipeline · Workspace · Job scheduler            │
├─────────────────────────────────────────────────────────────────┤
│  CAPABILITIES: Import · Parse · Analyze · Reason · Verify       │
├─────────────────────────────────────────────────────────────────┤
│  KNOWLEDGE: Graph engine · Query · Model · USIR                 │
├─────────────────────────────────────────────────────────────────┤
│  PLUGIN RUNTIME: Host · Registry · SDK · Sandbox                │
├─────────────────────────────────────────────────────────────────┤
│  FOUNDATION: Core · Schema · Store (CAS)                        │
└─────────────────────────────────────────────────────────────────┘
                              ▲
                              │ plugins attach here only
         ┌────────────────────┼────────────────────┐
         │ Importers │ Parsers │ Analyzers │ Reasoners │
         └────────────────────┴────────────────────┘
```

---

## 3. The Knowledge Model

S4MP materializes software understanding as a **layered multi-graph** with explicit provenance and time.

### 3.1 Graph Layers

| Layer | Nodes (examples) | Edges (examples) | Produced by |
|-------|------------------|------------------|-------------|
| **Physical** | Repo, Commit, Tree, Blob, Path | Contains, ParentOf, AtRevision | Importers |
| **Syntax** | FileUnit, Span, Token, AstNode | ChildOf, Covers | Parser plugins |
| **Semantic (USIR)** | Module, Symbol, Type, Callable, Value | Defines, References, Calls, Implements | Parsers + Linker |
| **Structural** | Package, Namespace, Dependency | DependsOn, Exports, Imports | Linker + Analyzers |
| **Architectural** | Boundary, Layer, Pattern, AntiPattern | Violates, ConformsTo, MapsTo | Architecture analyzers |
| **Feature** | Capability, Feature, EntryPoint | RealizedBy, ExposedVia | Feature extractors |
| **Requirements** | Requirement, Constraint, Test | Satisfies, TracesTo, VerifiedBy | Requirements plugins |
| **Quality** | Metric, Finding, Invariant, Certificate | Measures, Violates, Certifies | Analyzers + Verifiers |

### 3.2 Facts, Not Opinions

Every graph element carries:

- **Provenance** — `{ source_type, source_id, artifact_id, timestamp }`
- **Confidence** — `0.0–1.0` (deterministic facts = 1.0)
- **Lifecycle** — `proposed | accepted | rejected | superseded`
- **Schema version** — for forward compatibility

LLM outputs are always `proposed` until accepted by a verifier, human, or deterministic rule.

### 3.3 Time and Snapshots

- A **Snapshot** = immutable artifact manifest pointing at graph state at a revision.
- **Delta artifacts** link snapshots for incremental recomputation.
- Historical reasoning is a first-class query.

---

## 4. Universal Semantic IR (USIR)

USIR is S4MP's LLVM IR: the stable contract between parsers, linkers, and analyzers.

### 4.1 Core Entity Kinds

`Module`, `Package`, `Symbol`, `Type`, `Callable`, `Parameter`, `Field`, `GenericParam`, `Effect`, `Contract`, `Annotation`, `Location`, `Diagnostic`

### 4.2 Core Relation Kinds

`Defines`, `Declares`, `References`, `Calls`, `Implements`, `Extends`, `Contains`, `DependsOn`, `Annotates`, `AliasOf`, `Specializes`

### 4.3 Extension Mechanism

- Standard kinds are frozen slowly (like LLVM opcodes).
- Extension kinds use namespaced IDs (`com.example.custom/EdgeKind`).
- Unknown kinds are preserved opaquely.

---

## 5. Artifact Store

| Concept | Description |
|---------|-------------|
| **ArtifactId** | `blake3(content)` — immutable, deduplicated |
| **Artifact** | Typed blob: `{ kind, schema_version, payload }` |
| **Manifest** | Ordered list of ArtifactIds + metadata |
| **Workspace** | Mutable pointer to current manifest for a project |

All inter-crate and inter-plugin communication crosses the store.

---

## 6. Crate Map

> **Implementation:** see `crates/s4-*` in the workspace. Each crate has a README and trait-only public API.

### Tier 0 — Foundation

| Crate | Responsibility |
|-------|----------------|
| `s4-core` | IDs, errors, schema/API versioning, prelude |

### Tier 1 — Infrastructure

| Crate | Responsibility |
|-------|----------------|
| `s4-storage` | CAS artifact and manifest contracts |
| `s4-events` | Event bus and pub/sub contracts |
| `s4-plugin` | Plugin manifests and specialized plugin traits |
| `s4-project` | Workspace, config, lockfile, snapshot refs |
| `s4-graph` | Code graph nodes, edges, layers, queries |

### Tier 2 — Knowledge & Capabilities

| Crate | Responsibility |
|-------|----------------|
| `s4-parser` | USIR types and parse pipeline contracts |
| `s4-knowledge` | Facts, provenance, ontology, materializer |
| `s4-requirements` | Requirements graph and traceability |
| `s4-metrics` | Complexity and software metrics |
| `s4-analysis` | Architecture and feature extraction |
| `s4-llm` | LLM-agnostic reasoning (interfaces only) |

### Tier 3 — Quality & Planning

| Crate | Responsibility |
|-------|----------------|
| `s4-planner` | Refactoring plan contracts |
| `s4-verification` | Invariants, verifiers, acceptance workflows |
| `s4-certification` | Certificates, policies, issuer trait |

### Tier 4 — Surfaces

| Crate | Responsibility |
|-------|----------------|
| `s4-cli` | `s4` command-line interface |
| `s4-api` | HTTP/gRPC API contracts |
| `s4-ui` | Headless UI bridge for IDE/web frontends |

---

## 7. Ownership

| Domain | Owns |
|--------|------|
| Platform Core | `s4-core`, `s4-storage`, `s4-events`, `s4-project` |
| Knowledge Graph | `s4-graph`, `s4-knowledge`, `s4-parser` |
| Plugin Infrastructure | `s4-plugin` + future plugin packages |
| Language Ecosystem | Parser plugins (RFC for USIR schema) |
| Analysis & Quality | `s4-analysis`, `s4-metrics`, `s4-verification`, `s4-certification` |
| Intelligence (Interfaces) | `s4-llm`, `s4-planner` contracts only |
| Developer Experience | `s4-cli`, `s4-api`, `s4-ui` |

---

## 8. Plugin Boundaries

### Plugins MAY

- Read input artifacts from store
- Write new artifacts to store
- Emit diagnostics and metrics
- Register extension kinds (namespaced)

### Plugins MUST NOT

- Mutate existing artifacts
- Write directly to graph engine memory in production path
- Call other plugins directly
- Depend on a specific LLM provider (except reasoner plugins)
- Bypass provenance tagging

### Trust Tiers

| Tier | Sandbox |
|------|---------|
| Trusted first-party | In-process |
| Signed third-party | WASM or seccomp |
| Untrusted community | WASM, no network by default |

---

## 9. Data Flow

```
Repository URL
    │
    ▼
[Importer Plugin] ──► Physical Snapshot Artifact
    │
    ▼
[Parser Plugins] ──► Per-file Syntax + USIR Artifacts
    │
    ▼
[Linker Plugin] ──► Unified USIR + Symbol Table
    │
    ▼
[Graph Materializer] ──► Semantic + Structural Graph
    │
    ├──► [Analyzer Plugins] ──► Architectural / Feature / Quality
    ├──► [Requirements Plugins] ──► Requirements Graph
    ├──► [Query Engine] ──► Result Sets
    ├──► [Reasoner Plugins] ──► Proposal Artifacts
    └──► [Verifier Plugins] ──► Accepted Facts + Certificates
```

---

## 10. Dependency Rules

```
Tier 5 → Tier 4 → Tier 3 → Tier 2 → Tier 1 → Tier 0
```

**Forbidden:**

- Tier 0 → any outer tier
- `s4-graph` / `s4-knowledge` → `s4-llm`
- Any core crate → HTTP client, LLM SDK, language parser

Enforced by `deny.toml` and code review.

---

## 11. Architectural Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| ADR-001 | Knowledge model is the product | Survives LLM vendor churn |
| ADR-002 | Content-addressed artifact store | Reproducibility, caching, audit |
| ADR-003 | USIR as universal interchange | Language-agnostic analyzers |
| ADR-004 | Plugins at volatile boundaries | 5-year evolution without core rewrites |
| ADR-005 | LLM outputs always `proposed` | Prevents silent corruption of truth |
| ADR-006 | Layered graph projections | Multiple consumers, one source |
| ADR-007 | Declarative pipelines | Kubernetes-style operability |
| ADR-008 | Rust for core platform | Safety, performance, WASM path |
| ADR-009 | Schema-first APIs | Multi-language clients |
| ADR-010 | Query engine independent of AI | AI is optional accelerator |
| ADR-011 | No auto-apply refactors in core | Safety, trust, certification |
| ADR-012 | Blake3 for artifact IDs | Speed; intentionally not Git SHA-1 |

---

## 12. Future Bottlenecks

| Bottleneck | Mitigation |
|------------|------------|
| Graph size (large monorepos) | Sharded artifacts, lazy projection, hierarchical summaries |
| Cross-language linking | Linker v2 with external IDL artifacts |
| Incremental invalidation | Fine-grained artifact DAG |
| Query latency (IDE) | Materialized views, incremental indexes |
| Plugin sandbox overhead | Batch invocations, trusted fast path |
| USIR expressiveness | Opaque extension nodes |
| Schema migration debt | Migration tooling from day 1 |
| LLM context limits | Graph summarization, hierarchical context bundles |
| Plugin ecosystem fragmentation | Conformance suite + quality tiers |

---

## 13. Phased Delivery

| Phase | Deliverables |
|-------|--------------|
| **P0** | Tier 0 + store + schema + arch tests |
| **P1** | USIR + parse/link + git importer + Rust parser |
| **P2** | Graph + query + complexity analyzer |
| **P3** | Pipeline + workspace + CLI |
| **P4** | Reason interface + OpenAI-compatible reasoner plugin |
| **P5** | Requirements graph + traceability verifier |
| **P6** | WASM sandbox + registry |

---

## 14. Open Decisions

1. **Plugin Phase 1:** in-process native vs WASM from day one
2. **Schema encoding:** Protocol Buffers vs JSON Schema + CBOR
3. **Graph storage at scale:** embedded vs external vs custom over CAS
4. **Query language:** custom S4QL vs Datalog/Cypher subset

See `docs/adr/` for formal decision records as they are resolved.
