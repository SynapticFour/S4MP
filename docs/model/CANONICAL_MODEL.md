# S4MP Canonical Data Model
## Specification v0.1

> **Status:** Design baseline — no implementation  
> **Principle:** Language-agnostic semantics first; language mappings are projections, not truth.

---

## 1. Purpose of This Document

This document defines the **canonical data model** for the SynapticFour Method Platform (S4MP). Every crate, plugin, analyzer, and surface builds upon these entities and their relationships.

The model is designed to:

1. Represent **software systems as durable knowledge**, not transient parse output.
2. Remain **valid across programming languages** by preferring universal abstractions.
3. Support **provenance, time, confidence, and lifecycle** on every fact.
4. Allow **additive extension** without breaking consumers.
5. Serialize identically for storage, transport, and audit.

---

## 2. Meta-Model (Applies to Every Entity)

Before entity definitions, these cross-cutting constructs apply platform-wide.

### 2.1 Entity Envelope

Every persisted entity is wrapped in an **EntityEnvelope**:

| Field | Type | Purpose |
|-------|------|---------|
| `id` | `EntityId` | Stable identifier within a snapshot scope |
| `kind` | `EntityKind` | Canonical or extension kind |
| `schema_version` | `SchemaVersion` | `{ major, minor }` of this entity's schema |
| `payload` | object | Kind-specific fields |
| `provenance` | `Provenance` | Who/what/when produced this entity |
| `confidence` | `Confidence` | `0.0–1.0`; deterministic facts = `1.0` |
| `lifecycle` | `FactLifecycle` | `proposed \| accepted \| rejected \| superseded` |
| `extensions` | map | Namespaced opaque attributes |

**Why:** Separates *identity* and *metadata about belief* from *domain payload*. Enables certification, AI proposals, and historical replay.

### 2.2 Identifiers

| ID Type | Scope | Format | Stability |
|---------|-------|--------|-----------|
| `ProjectId` | Global platform | URI or UUID string | Stable across workspace lifetime |
| `ArtifactId` | Content-addressed store | Blake3 hex (32 bytes) | Immutable per content |
| `SnapshotId` | Project history | `ArtifactId` of manifest | Immutable |
| `EntityId` | Within snapshot | `{ snapshot, kind, local_key }` | Stable within snapshot; may change across snapshots |
| `LocalKey` | Parser/linker assigned | Opaque string or u64 | Used until cross-snapshot identity resolved |
| `SymbolKey` | Cross-snapshot identity | Qualified name + module path + language hint | Best-effort stable reference |
| `ExtensionKindId` | Ontology registry | Namespaced URI string | Permanent once registered |

**Why:** Multiple ID layers prevent coupling storage layout to semantic identity, while enabling incremental recomputation and audit.

### 2.3 Provenance

```
Provenance {
  source_type:  import | parse | link | analysis | human | reasoner | verifier | transform
  source_id:    string          // plugin name, user id, pipeline stage
  artifact_id:  ArtifactId      // CAS blob that produced this entity
  timestamp:    ISO-8601
  parent_ids:   EntityId[]      // upstream entities this was derived from
}
```

**Why:** Certification and debugging require tracing every fact to an immutable source artifact.

### 2.4 Serialization

| Aspect | Decision |
|--------|----------|
| **Canonical encoding** | JSON with deterministic key ordering for hashing; CBOR for compact storage (ADR pending) |
| **Envelope** | All entities serialize as `EntityEnvelope` |
| **Graphs** | Entities + separate edge records (not nested adjacency by default) |
| **Large payloads** | Inline below size threshold; otherwise `ArtifactId` reference |
| **Unknown fields** | Preserved opaquely (forward compatibility) |

**Why:** JSON for debuggability and schema evolution; content-addressing for deduplication and reproducibility.

### 2.5 Versioning

| Level | Mechanism |
|-------|-----------|
| **Schema version** | `{ major, minor }` on every envelope. Major = breaking payload change. Minor = additive fields. |
| **Snapshot version** | Immutable manifest chain; parent pointer per snapshot |
| **Entity supersession** | New envelope with `lifecycle: superseded` on old; `parent_ids` link |
| **Migration** | Explicit transform artifacts produce new `ArtifactId`; old never mutated |

**Why:** Five-year evolution requires reading old snapshots without silent semantic drift.

### 2.6 Extension Strategy

1. **Standard kinds** — frozen slowly; listed in this document.
2. **Extension kinds** — `ExtensionKindId` = `"com.vendor/kind-name"`.
3. **Extension payload** — `extensions["com.vendor/attr"] = value`.
4. **Unknown kinds** — stored and forwarded opaquely; consumers must not drop.
5. **Registration** — plugins declare extensions in manifest; ontology registry records them.

**Why:** Languages and domains evolve faster than core schema. Extensions prevent core bloat.

### 2.7 Ownership (Platform-Wide)

| Domain | Owns entity kinds |
|--------|-------------------|
| **Workspace** | Project, Snapshot refs |
| **Import plugins** | Repository, File (physical) |
| **Parser plugins** | Language metadata, Module, Symbol, Callable, TypeDefinition, Contract, Call (syntax-derived) |
| **Linker** | Dependency, cross-module resolution |
| **Knowledge materializer** | Concept, Domain mappings |
| **Requirements plugins / humans** | Requirement |
| **Analysis plugins** | Metric, Feature, Flow, Architecture views |
| **Verification** | Verification, Invariant results |
| **Reasoner plugins** | Proposed entities (always `proposed` lifecycle) |
| **Transform plugins** | Transformation (plans; never silent apply) |

---

## 3. Layer Model

Entities belong to conceptual layers. Layers are **views**, not separate databases.

```
┌─────────────────────────────────────────────────────────────┐
│  INTENT: Requirement, Feature, Flow                        │
├─────────────────────────────────────────────────────────────┤
│  KNOWLEDGE: Concept, Domain                                  │
├─────────────────────────────────────────────────────────────┤
│  QUALITY: Metric, Architecture, Verification, Transformation│
├─────────────────────────────────────────────────────────────┤
│  SEMANTIC: Module, Symbol, Callable, TypeDefinition,       │
│            Contract, Call, Dependency                        │
├─────────────────────────────────────────────────────────────┤
│  PHYSICAL: Project, Repository, Language, File             │
└─────────────────────────────────────────────────────────────┘
```

**Why:** Layers express *epistemic strength* and *derivation order*, not ownership silos.

---

## 4. Entity Definitions

---

### 4.1 Project

| Attribute | Value |
|-----------|-------|
| **Purpose** | Root container for all knowledge about one software effort under analysis |
| **Ownership** | Workspace / `s4-project` |
| **Relationships** | Has many `Repository`; has many `Snapshot`; configures plugins |
| **Identifier** | `ProjectId` (global, human-chosen or assigned) |
| **Serialization** | `kind: "project"`, payload: `{ name, config_ref, lockfile_ref }` |
| **Versioning** | Config evolves; snapshots immutable |
| **Extension** | `extensions["com.team/workflow"]` for custom metadata |

**Why it exists:** S4MP analyzes *projects*, not anonymous file trees. Projects bind configuration, history, and certification scope.

**Payload (conceptual):**
```
Project {
  name: string
  root_path: string
  active_snapshot: SnapshotId?
  plugin_config: PluginRef[]
}
```

---

### 4.2 Repository

| Attribute | Value |
|-----------|-------|
| **Purpose** | A version-controlled or importable source of truth for code |
| **Ownership** | Import plugins |
| **Relationships** | Belongs to `Project`; contains `File` at `Snapshot`; may link to remote URI |
| **Identifier** | `EntityId` within snapshot + optional `remote_uri` |
| **Serialization** | `kind: "repository"`, payload: `{ uri, vcs_kind, revision, root_tree }` |
| **Versioning** | Each import produces new snapshot delta |
| **Extension** | `vcs_kind: "extension:fossil"` etc. |

**Why it exists:** Code enters the platform through repositories. Separating Repository from Project supports monorepos, polyrepos, and multi-source projects.

**Language-agnostic note:** Repository describes *provenance of bytes*, not semantics.

---

### 4.3 Language

| Attribute | Value |
|-----------|-------|
| **Purpose** | Metadata label for how a file or symbol should be parsed and interpreted |
| **Ownership** | Parser plugins (declaration); platform registry (known IDs) |
| **Relationships** | Annotates `File`, `Module`, `Symbol`; selects parser plugin |
| **Identifier** | `LanguageId` string (e.g. `"rust"`, `"typescript"`, `"protobuf"`) |
| **Serialization** | `kind: "language"` or embedded field `language_id` on File/Module |
| **Versioning** | Language ID stable; parser mappings version separately |
| **Extension** | `"ext:graphql-schema"`, `"ext:wasm"` |

**Why it exists:** Parsing is language-specific; **knowledge is not**. Language is metadata, never an ontology root.

**Critical rule:** No analysis logic depends on Language except to select a plugin.

---

### 4.4 File

| Attribute | Value |
|-----------|-------|
| **Purpose** | Physical or logical source unit containing text or binary content |
| **Ownership** | Import plugins (discovery); store (content via `ArtifactId`) |
| **Relationships** | Belongs to `Repository`; contains `Module`/`Symbol` roots; referenced by `Metric` |
| **Identifier** | Path within repository + content hash at snapshot |
| **Serialization** | `kind: "file"`, payload: `{ path, content_hash, language_id?, size, encoding }` |
| **Versioning** | Content hash changes → new file entity; path rename = transform edge |
| **Extension** | Generated file flags, build artifact markers |

**Why it exists:** All semantic knowledge ultimately anchors to locatable source. Files are the bridge between bytes and meaning.

---

### 4.5 Module

| Attribute | Value |
|-----------|-------|
| **Purpose** | Namespace boundary grouping symbols — package, crate, namespace, compilation unit |
| **Ownership** | Parser + linker |
| **Relationships** | Contains `Symbol`, `Callable`, `TypeDefinition`, `Contract`; source is `File`(s); target of `Dependency` |
| **Identifier** | Qualified module path within snapshot |
| **Serialization** | `kind: "module"`, payload: `{ qualified_name, file_ids[], visibility, language_id }` |
| **Versioning** | Qualified name may change; track via `Transformation` rename records |
| **Extension** | Build system module IDs (Maven coords, Go module path) |

**Why it exists:** Software is organized hierarchically. Module is the **universal** boundary — not "class file" or "crate" specifically.

**Language projections:**

| Language | Maps to Module |
|----------|----------------|
| Rust | crate root, mod |
| Java | package |
| Python | package, module |
| TypeScript | file, namespace |
| C | translation unit (with caveats) |

---

### 4.6 Symbol

| Attribute | Value |
|-----------|-------|
| **Purpose** | Named program element addressable within a module — the universal identity carrier |
| **Ownership** | Parser + linker |
| **Relationships** | Defined in `Module`; specializes into `Callable`, `TypeDefinition`, `Contract`; referenced by `Call`, `Dependency`, `Requirement` |
| **Identifier** | `SymbolKey`: `{ module_path, name, kind_hint, disambiguator? }` |
| **Serialization** | `kind: "symbol"`, payload: `{ name, qualified_name, role, definition_site }` |
| **Versioning** | Renames tracked as `Transformation` |
| **Extension** | Language-specific attributes (generics, async, etc.) in extensions bag |

**Why it exists:** Functions, classes, and traits are **roles** a symbol plays — not separate ontological roots. Symbol enables uniform referencing in requirements, metrics, and flows.

**Role enum (not separate top-level entities for identity):**
```
SymbolRole: callable | type_definition | contract | value | macro | unknown
```

---

### 4.7 Callable

| Attribute | Value |
|-----------|-------|
| **Purpose** | Invokable behavior — function, method, procedure, lambda, constructor |
| **Ownership** | Parser (extraction); analyzer (metrics) |
| **Relationships** | Specialization of `Symbol`; source of `Call` edges; measured by `Metric`; traced by `Requirement` |
| **Identifier** | Same as parent `Symbol` + `EntityId` |
| **Serialization** | `kind: "callable"`, payload: `{ symbol_ref, parameters[], return_type_ref?, effects[] }` |
| **Versioning** | Signature changes produce superseded entity |
| **Extension** | Async, throws, purity, visibility |

**Why it exists:** Calls, complexity, and feature entry points attach to **behavior**. Callable is the universal behavior node — "Function" is the imperative/OOP projection.

**Language projections:**

| Language | Callable examples |
|----------|-------------------|
| Rust | fn, method, closure |
| Java | method, constructor |
| Python | def, lambda |
| Haskell | function |
| SQL | stored procedure |

---

### 4.8 TypeDefinition

| Attribute | Value |
|-----------|-------|
| **Purpose** | Named type or data structure definition — aggregates state and structure |
| **Ownership** | Parser |
| **Relationships** | Specialization of `Symbol`; related to `Contract` via `implements`; referenced by `Callable` signatures |
| **Identifier** | Parent `Symbol` |
| **Serialization** | `kind: "type_definition"`, payload: `{ symbol_ref, members[], type_kind, generics[] }` |
| **Versioning** | Member changes supersede |
| **Extension** | Class, struct, enum, union, alias — as `type_kind` or extension |

**Why it exists:** "Class" is one language's view of a type definition. S4MP uses **TypeDefinition** to cover struct, class, enum, record, typedef uniformly.

**Language projections:**

| Language | TypeDefinition |
|----------|----------------|
| Java/C# | class, interface (see Contract), enum |
| Rust | struct, enum, union |
| TypeScript | class, interface, type alias |
| Go | struct |

---

### 4.9 Contract

| Attribute | Value |
|-----------|-------|
| **Purpose** | Behavioral or structural surface without full implementation — interface, trait, protocol, abstract API |
| **Ownership** | Parser |
| **Relationships** | Specialization of `Symbol`; `TypeDefinition` may `implement` Contract; `Dependency` may target Contract |
| **Identifier** | Parent `Symbol` |
| **Serialization** | `kind: "contract"`, payload: `{ symbol_ref, required_methods[], properties[], contract_kind }` |
| **Versioning** | Contract evolution tracked for breaking-change analysis |
| **Extension** | Trait bounds, protocol extensions |

**Why it exists:** **Trait** and **Interface** are language keywords for the same concept: an **agreement** about behavior/shape. Contract enables architecture extraction and dependency inversion analysis across languages.

**Language projections:**

| Language | Contract |
|----------|----------|
| Rust | trait |
| Java/C# | interface |
| TypeScript | interface |
| Go | interface |
| Swift | protocol |

---

### 4.10 Call

| Attribute | Value |
|-----------|-------|
| **Purpose** | Resolved or unresolved invocation from one callable to another |
| **Ownership** | Parser (direct); linker (cross-module); analysis (dynamic/refined) |
| **Relationships** | Edge: `Callable` → `Callable` (or `Symbol`); contributes to `Flow` |
| **Identifier** | `{ caller_ref, callee_ref, call_site_location }` |
| **Serialization** | `kind: "call"`, edge record: `{ from, to, resolution: static|dynamic|unknown, site }` |
| **Versioning** | Resolution confidence may improve across analysis passes |
| **Extension** | Virtual dispatch, async await boundary |

**Why it exists:** Call graph is fundamental to impact analysis, refactoring, and architecture. Separating Call from Dependency captures *runtime behavior* vs *compile-time coupling*.

---

### 4.11 Dependency

| Attribute | Value |
|-----------|-------|
| **Purpose** | Compile-time, build-time, or logical coupling between modules, symbols, or external systems |
| **Ownership** | Parser + linker + import analysis |
| **Relationships** | Edge: Module/Symbol/Contract → Module/Symbol/Contract/External; distinct from `Call` |
| **Identifier** | `{ from_ref, to_ref, dependency_kind }` |
| **Serialization** | `kind: "dependency"`, edge: `{ from, to, kind: imports|implements|extends|uses|external, version_constraint? }` |
| **Versioning** | Dependency graphs compared across snapshots for drift |
| **Extension** | Package manager coords, license, scope |

**Why it exists:** Architecture and certification require knowing *what depends on what* even when no direct call exists (interfaces, configs, DI).

---

### 4.12 Concept

| Attribute | Value |
|-----------|-------|
| **Purpose** | Language-neutral abstract idea present in the software domain — not tied to a symbol name |
| **Ownership** | Knowledge materializer, humans, reasoners (proposed) |
| **Relationships** | Maps to `Symbol`, `Feature`, `Requirement`, `Domain`; many-to-many |
| **Identifier** | `ConceptId` (URI or platform-assigned) |
| **Serialization** | `kind: "concept"`, payload: `{ label, description, aliases[] }` |
| **Versioning** | Concepts merge/split via explicit knowledge curation |
| **Extension** | Domain-specific ontologies (DDD, enterprise glossary) |

**Why it exists:** Code names lie. **Concept** is the knowledge-layer entity that survives renames and multilingual teams — "UserAuthentication" as idea vs `AuthService` as symbol.

---

### 4.13 Domain

| Attribute | Value |
|-----------|-------|
| **Purpose** | Bounded context or problem space grouping concepts, modules, and requirements |
| **Ownership** | Humans, architecture analyzers, reasoners (proposed) |
| **Relationships** | Contains `Concept`, `Feature`, `Requirement`; maps to `Module` subsets; defines `Architecture` boundaries |
| **Identifier** | `DomainId` |
| **Serialization** | `kind: "domain"`, payload: `{ name, description, parent_domain?, concept_refs[] }` |
| **Versioning** | Domain boundaries evolve; tracked as accepted facts |
| **Extension** | DDD context map metadata |

**Why it exists:** Large systems decompose by domain. Domain links **intent** (requirements) to **structure** (modules) without language coupling.

---

### 4.14 Requirement

| Attribute | Value |
|-----------|-------|
| **Purpose** | Formal or informal statement of needed system behavior or constraint |
| **Ownership** | Humans, requirements plugins, reasoners (proposed) |
| **Relationships** | Traces to `Symbol`, `Callable`, `Feature`, `Test`; belongs to `Domain`; verified by `Verification` |
| **Identifier** | `RequirementId` (often external: JIRA key, req ID) |
| **Serialization** | `kind: "requirement"`, payload: `{ external_id?, statement, priority, type: functional|non_functional|constraint }` |
| **Versioning** | Requirements supersede; full trace history preserved |
| **Extension** | Regulatory tags, safety integrity level |

**Why it exists:** Certification and traceability require linking *what was asked for* to *what was built*. Requirement is the intent anchor.

**Edge kinds:**
```
Requirement --[satisfies]--> Symbol | Feature
Requirement --[verified_by]--> Verification
Requirement --[belongs_to]--> Domain
```

---

### 4.15 Metric

| Attribute | Value |
|-----------|-------|
| **Purpose** | Quantitative or boolean measurement of code or architecture |
| **Ownership** | Analysis plugins, metrics collectors |
| **Relationships** | Attached to `Symbol`, `Callable`, `Module`, `File`, `Architecture` view |
| **Identifier** | `{ target_ref, metric_kind, snapshot }` |
| **Serialization** | `kind: "metric"`, payload: `{ target_ref, metric_kind, value, unit? }` |
| **Versioning** | Metrics recomputed per snapshot; stored as new entities |
| **Extension** | Custom metric definitions |

**Why it exists:** Complexity, coupling, coverage, and quality gates require **persistent measurements** comparable over time.

**Standard metric kinds:**
```
cyclomatic_complexity | cognitive_complexity | coupling | loc | coverage | depth | fan_in | fan_out
```

---

### 4.16 Feature

| Attribute | Value |
|-----------|-------|
| **Purpose** | User-visible or system capability realized by code |
| **Ownership** | Feature extractors, humans, reasoners (proposed) |
| **Relationships** | Maps to `Callable` entry points, `Module`, `Flow`; satisfies `Requirement`; belongs to `Domain` |
| **Identifier** | `FeatureId` |
| **Serialization** | `kind: "feature"`, payload: `{ name, description, entry_point_refs[] }` |
| **Versioning** | Feature boundaries shift; link to snapshots |
| **Extension** | Product management IDs |

**Why it exists:** Architecture and planning operate at the **capability** level, not the function level. Feature bridges product language and code.

---

### 4.17 Flow

| Attribute | Value |
|-----------|-------|
| **Purpose** | Ordered or graph-shaped path through behavior — control flow, data flow, user journey, request lifecycle |
| **Ownership** | Analysis plugins, reasoners (proposed) |
| **Relationships** | Composed of `Call` edges, `Symbol` nodes; may realize `Feature`; validated by `Verification` |
| **Identifier** | `FlowId` |
| **Serialization** | `kind: "flow"`, payload: `{ name, flow_kind, steps: FlowStep[] }` |
| **Versioning** | Flows refined as call graph resolution improves |
| **Extension** | Sequence diagrams, async boundaries |

**Why it exists:** Understanding *sequences* and *paths* requires more than a call graph. Flow captures narrative structure for features, security review, and certification.

**Flow kinds:**
```
control | data | user | request | event | extension:*
```

**FlowStep:**
```
{ order, node_ref, edge_ref?, annotation? }
```

---

### 4.18 Architecture

| Attribute | Value |
|-----------|-------|
| **Purpose** | **Composite view** describing structure, boundaries, patterns, and layers — not a single atomic entity |
| **Ownership** | Architecture analyzers, humans |
| **Relationships** | Annotates `Module`, `Domain`, `Dependency` graphs; produces `Metric` and findings |
| **Identifier** | `ArchitectureViewId` per snapshot + view name |
| **Serialization** | `kind: "architecture_view"`, payload: `{ name, boundaries[], patterns[], layers[] }` |
| **Versioning** | Views recomputed; compared across snapshots |
| **Extension** | C4, arc42, custom viewpoints |

**Why it exists:** "Architecture" is a *perspective* on the graph, not a node in the code. Modeling it explicitly enables layer violation detection and pattern conformance.

**Sub-structures (embedded in view, not separate roots):**

| Sub-structure | Purpose |
|---------------|---------|
| `Boundary` | Named group of modules with coupling rules |
| `Layer` | Ordered architectural tier (presentation, domain, infra) |
| `Pattern` | Detected or declared pattern instance |
| `AntiPattern` | Violation record with severity |

---

### 4.19 Verification

| Attribute | Value |
|-----------|-------|
| **Purpose** | Record of evaluating invariants, tests, or policies against knowledge |
| **Ownership** | Verifier plugins, CI integration |
| **Relationships** | Evaluates `Requirement`, `Invariant`, `Architecture` rules; produces pass/fail + evidence |
| **Identifier** | `VerificationId` + `ArtifactId` of result |
| **Serialization** | `kind: "verification"`, payload: `{ rule_set, passed, violations[], evidence_refs[] }` |
| **Versioning** | Each run is immutable; certificate links to verification |
| **Extension** | CI run IDs, test framework metadata |

**Why it exists:** Certification requires **replayable proof** that checks ran against a specific snapshot.

**Related (embedded):**
```
Invariant { name, expression, description }
Violation { rule_ref, entity_ref, message }
```

---

### 4.20 Transformation

| Attribute | Value |
|-----------|-------|
| **Purpose** | Proposed or applied change to code or knowledge — refactor plan, rename, migration, AI suggestion |
| **Ownership** | Planner, reasoners (proposed), transform plugins (apply) |
| **Relationships** | Targets `Symbol`, `File`, `Module`; triggered by `Metric`/`Architecture` findings; validated by `Verification` |
| **Identifier** | `TransformationId` |
| **Serialization** | `kind: "transformation"`, payload: `{ title, steps[], risk, status: proposed|approved|applied|rejected }` |
| **Versioning** | Transformations never mutate history; application creates new snapshot |
| **Extension** | IDE edit formats, patch artifacts |

**Why it exists:** S4MP **plans and certifies change**; it does not silently rewrite code. Transformation makes intent explicit and auditable.

**TransformationStep kinds:**
```
rename | move | extract | inline | add_test | edit | manual_review
```

---

## 5. Relationship Summary

```
Project
  └── Repository
        └── File
              └── Module
                    └── Symbol ──┬── Callable
                                 ├── TypeDefinition
                                 └── Contract

Callable ──[call]──> Callable
Module/Symbol ──[dependency]──> Module/Symbol | External

Concept ──[maps_to]──> Symbol | Feature
Domain ──[contains]──> Concept | Requirement | Feature

Requirement ──[traces_to]──> Symbol | Feature | Callable
Feature ──[realized_by]──> Callable | Module
Flow ──[composed_of]──> Call | Symbol

Metric ──[measures]──> Symbol | Callable | Module | File
ArchitectureView ──[groups]──> Module | Boundary | Pattern
Verification ──[evaluates]──> Requirement | Invariant | ArchitectureView
Transformation ──[targets]──> Symbol | File | Module
```

---

## 6. Language-Specific Projection Table

Language keywords map **into** canonical entities — never the reverse.

| Language concept | Canonical entity | Notes |
|------------------|------------------|-------|
| function, method, fn, def | **Callable** | |
| class, struct, enum | **TypeDefinition** | `type_kind` discriminates |
| interface, trait, protocol | **Contract** | |
| module, package, crate, namespace | **Module** | |
| import, use, require | **Dependency** | |
| invoke, call | **Call** | |
| Any named element | **Symbol** | Identity root |

**Rule:** Parsers emit canonical entities + language extensions. Analyzers consume canonical entities only.

---

## 7. Lifecycle and Epistemic Rules

| Source | Default lifecycle | Default confidence |
|--------|-------------------|-------------------|
| Import / Parse / Link | `accepted` | `1.0` |
| Deterministic analysis | `accepted` | `1.0` |
| Heuristic analysis | `accepted` | `0.7–0.95` |
| Human entry | `accepted` | `1.0` |
| LLM / reasoner | **`proposed`** | model-assigned |
| Verification pass | promotes to `accepted` | `1.0` |
| Verification fail | `rejected` or remains `proposed` | — |

**Why:** The knowledge model is the product. Truth is earned, not generated.

---

## 8. Mapping to S4MP Crates

| Model area | Primary crate |
|------------|---------------|
| Project, Snapshot | `s4-project` |
| File, Repository (physical) | `s4-storage` + import plugins |
| Language, Module, Symbol, Callable, TypeDefinition, Contract, USIR | `s4-parser` |
| Call, Dependency, graph layers | `s4-graph` |
| Concept, Domain, Fact, Provenance | `s4-knowledge` |
| Requirement, traceability | `s4-requirements` |
| Metric | `s4-metrics` |
| Feature, Architecture, Flow | `s4-analysis` |
| Verification | `s4-verification` |
| Transformation | `s4-planner` |
| LLM proposals | `s4-llm` (proposal envelope) |
| Certification | `s4-certification` |

---

## 9. Open Decisions

1. **SymbolKey stability algorithm** across snapshots (AST hash vs qualified name vs LSP index).
2. **External entity** representation for third-party dependencies not in repository.
3. **Flow** storage: inline steps vs graph-native path queries.
4. **Architecture view** merge strategy when multiple analyzers disagree.
5. **Concept merge** UX for human curation.

---

## 10. Summary

The S4MP canonical model centers on:

1. **Symbol** as universal identity
2. **Callable / TypeDefinition / Contract** as semantic roles
3. **Call / Dependency** as behavioral vs structural coupling
4. **Concept / Domain / Requirement / Feature** as intent layer
5. **Metric / Architecture / Verification / Transformation** as quality and evolution layer
6. **EntityEnvelope** with provenance and lifecycle on everything

Language-specific terms are **projections**, not primitives. Everything serializes through versioned envelopes into the content-addressed store. Extensions carry what the core should not know.

This model is the foundation upon which all crates, plugins, and surfaces build.
