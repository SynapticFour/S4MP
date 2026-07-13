# Requirements Graph
## Architecture Specification v0.1

> **Status:** Design baseline — no implementation  
> **Contract crate:** `s4-requirements`  
> **Related:** [Canonical Model](../model/CANONICAL_MODEL.md), [Software Knowledge Graph](../knowledge/SOFTWARE_KNOWLEDGE_GRAPH.md), [UCG](../graph/UNIVERSAL_CODE_GRAPH.md)

---

## 1. Purpose

The **Requirements Graph** converts human intent — especially **Statements of Work (SOW)** and related contract documents — into **machine-readable, traceable requirements** that bind to code, tests, verification, and the Software Knowledge Graph.

| Goal | Mechanism |
|------|-----------|
| SOW → structured requirements | Parse, extract, normalize, curate |
| Every requirement traceable | Mandatory trace edges or explicit `untraceable` waiver |
| Multi-type constraints | Typed requirement kinds with schemas |
| Certification-ready | Version history + confidence + verification links |
| Living document | Supersession chain; diff across SOW revisions |

**Why a separate graph:** Requirements are **contractual intent** — authored, negotiated, versioned, and legally meaningful. They are not inferred business concepts (SKG) nor syntactic structure (UCG). Merging them loses auditability.

---

## 2. Position in the Platform

```
┌─────────────────────────────────────────────────────────────┐
│  SOURCES: SOW · RFP · OpenAPI · NFR docs · human entry       │
└────────────────────────────┬────────────────────────────────┘
                             │ parse / import
                             ▼
┌─────────────────────────────────────────────────────────────┐
│              REQUIREMENTS GRAPH (intent / contract)          │
│  Business · API · Performance · Security · Architecture · AC │
└───────┬─────────────────┬─────────────────┬─────────────────┘
        │ traces_to       │ satisfies       │ verified_by
        ▼                 ▼                 ▼
┌───────────────┐  ┌───────────────┐  ┌───────────────────────┐
│      SKG      │  │     UCG       │  │  Tests · Verification│
│   (meaning)   │  │  (structure)  │  │      (evidence)       │
└───────────────┘  └───────────────┘  └───────────────────────┘
```

| Graph | Question |
|-------|----------|
| **Requirements** | What must the system do or satisfy? |
| **SKG** | What does the system mean? |
| **UCG** | How is it built? |

---

## 3. Requirement Taxonomy

### 3.1 Requirement Kinds

| Kind | ID prefix | Purpose | Typical source |
|------|-----------|---------|----------------|
| **BusinessRequirement** | `BR-` | Stakeholder outcome or capability | SOW prose |
| **ApiContract** | `API-` | Interface obligation (endpoint, schema, SLA) | OpenAPI, SOW API section |
| **PerformanceConstraint** | `PERF-` | Latency, throughput, capacity | SOW NFR |
| **SecurityConstraint** | `SEC-` | Auth, encryption, compliance controls | SOW security section |
| **ArchitecturalConstraint** | `ARCH-` | Layering, technology, integration rules | SOW architecture, ADRs |
| **AcceptanceCriterion** | `AC-` | Testable condition for sign-off | SOW acceptance, user stories |

All kinds share a common **`Requirement`** envelope; kind-specific payload in typed extension.

### 3.2 Requirement Node

```
RequirementNode {
  id:              RequirementId      // platform-stable + external ref
  kind:            RequirementKind
  title:           string
  statement:       string             // normative text (SHALL/SHOULD)
  priority:        must | should | may
  status:          draft | active | superseded | waived
  source:          SourceRef          // SOW section, ticket, human
  parent_id:       Option<RequirementId>  // decomposition tree
  acceptance_criteria: RequirementId[]    // AC-* children
  envelope:        EntityEnvelope     // provenance, confidence, lifecycle
  kind_payload:    KindPayload        // typed per kind
}
```

### 3.3 Kind-Specific Payloads

**BusinessRequirement:**
```
{ outcome: string, stakeholders: string[], domain_hint: Option<DomainId> }
```

**ApiContract:**
```
{ spec_ref: ArtifactId,           // OpenAPI/Protobuf artifact
  operation_id: string?,
  method: string?, path: string?,
  schema_constraints: json }
```

**PerformanceConstraint:**
```
{ metric: string, operator: lte|gte|eq, value: number, unit: string, scope: string }
```

**SecurityConstraint:**
```
{ control_type: auth|encrypt|audit|privacy|...,
  standard: Option<string>,        // OWASP, SOC2, ISO27001
  control_id: Option<string> }
```

**ArchitecturalConstraint:**
```
{ rule: string,
  scope: layer|module|integration|technology,
  enforcement: static|review|runtime }
```

**AcceptanceCriterion:**
```
{ given: string, when: string, then: string,   // Gherkin optional
  test_ref: Option<TestRef> }
```

### 3.4 Requirement Hierarchy

SOW decomposition is a **tree** (with optional cross-links):

```
BR-001 Order Management (BusinessRequirement)
  ├── BR-001.1 Create Order
  │     ├── AC-001.1 Order appears in admin dashboard
  │     └── AC-001.2 Confirmation email sent
  ├── API-001 POST /orders OpenAPI contract
  ├── PERF-001 Checkout p99 < 500ms
  ├── SEC-001 Customer PII encrypted at rest
  └── ARCH-001 Billing module must not depend on UI layer
```

**Edges:** `decomposes_to` (parent → child), `refines` (BR → AC).

---

## 4. Edge Taxonomy (Traceability)

Traceability is **mandatory**. Every active requirement must have at least one of:

- `satisfies` / `implemented_by` → UCG or SKG
- `verified_by` → test or verification run
- `waived` → explicit waiver record with approver

| Edge | From → To | Meaning |
|------|-----------|---------|
| `decomposes_to` | Requirement → Requirement | SOW hierarchy |
| `depends_on` | Requirement → Requirement | Prerequisite |
| `conflicts_with` | Requirement → Requirement | Detected contradiction |
| `traces_to` | Requirement → SKG Concept/Domain | Intent ↔ meaning |
| `satisfies` | UCG Symbol/Feature → Requirement | Code fulfills (inverse: `satisfied_by`) |
| `implements` | ApiContract → UCG Symbol/OpenAPI op | Contract ↔ code |
| `verified_by` | Requirement → TestCase | Automated/manual test |
| `verified_by` | Requirement → VerificationRun | Batch verification |
| `evidenced_by` | Requirement → ArtifactId | SOW clause, design doc |
| `waived` | WaiverRecord → Requirement | Intentional non-trace with approval |

**Trace link metadata:**
```
TraceLink {
  requirement_id: RequirementId
  target:           TraceTarget      // UCG node | SKG node | Test | Verification
  link_kind:        TraceLinkKind
  confidence:       f32
  lifecycle:        proposed | accepted | rejected
  provenance:       Provenance
  snapshot_scope:   Option<SnapshotId>  // code snapshot when link valid
}
```

**Why bidirectional queries matter:** Certification asks both "what code satisfies SEC-001?" and "what requirements does this module satisfy?"

---

## 5. Parsing — SOW to Machine-Readable

### 5.1 Ingestion Pipeline

```
SOW document (DOCX, PDF, Markdown, HTML)
    │
    ▼
DocumentImporter plugin → text + structure artifact (sections, tables)
    │
    ▼
SowParser plugin
    ├── Structure detection (headings, numbered lists, SHALL/SHOULD)
    ├── Requirement candidate extraction
    └── Kind classification (BR, PERF, SEC, …)
    │
    ▼
RequirementProposal artifact (lifecycle: proposed)
    │
    ▼
Human review OR auto-accept (deterministic OpenAPI path)
    │
    ▼
RequirementsGraph snapshot
```

### 5.2 Parsing Strategies by Source

| Source | Parser | Confidence |
|--------|--------|------------|
| **Structured SOW template** | Rule-based section templates | 0.85–0.95 |
| **Unstructured prose** | LLM extraction + SHALL/SHOULD patterns | 0.5–0.75 (proposed) |
| **OpenAPI / AsyncAPI** | Schema import → ApiContract nodes | 0.95–1.0 |
| **Gherkin feature files** | Direct → AcceptanceCriterion | 0.9 |
| **Existing YAML/ReqIF** | Schema import | 1.0 |
| **JIRA / Linear export** | Ticket → Requirement mapping | 0.9 |

### 5.3 SOW Section Patterns

Detect normative language (ISO/IEC directive style):

| Pattern | Priority | Example |
|---------|----------|---------|
| **shall** / **must** | `must` | "The system shall authenticate users via MFA" |
| **should** | `should` | "Response should cache for 60 seconds" |
| **may** | `may` | "Admin may export reports" |

Section headers map to kinds:

| Header keywords | Kind |
|-----------------|------|
| "Functional", "Business", "Scope" | BusinessRequirement |
| "API", "Interface", "Integration" | ApiContract |
| "Performance", "Scalability", "Availability" | PerformanceConstraint |
| "Security", "Privacy", "Compliance" | SecurityConstraint |
| "Architecture", "Technical constraints" | ArchitecturalConstraint |
| "Acceptance", "Success criteria" | AcceptanceCriterion |

### 5.4 Parser Plugin Trait

```rust
trait RequirementParser: Plugin {
  fn parse_document(&self, ctx: &mut InvokeContext, doc: DocumentRef) -> Result<RequirementProposal>;
  fn parse_openapi(&self, ctx: &mut InvokeContext, spec: ArtifactId) -> Result<Vec<RequirementNode>>;
  fn classify_kind(&self, section: &DocumentSection) -> RequirementKind;
}
```

**Core never parses SOW** — only validates schema and stores artifacts.

### 5.5 LLM Role in Parsing

- Input: SOW section text + document outline + existing requirement IDs (avoid duplicates)
- Output: `RequirementProposal` with structured nodes + suggested hierarchy
- **Always proposed** until human or verifier accepts
- Human reviewer sees side-by-side: original SOW clause ↔ extracted requirement

---

## 6. Manual Editing

### 6.1 Authoring Surfaces

| Surface | Actions |
|---------|---------|
| **CLI** | `s4 req add`, `s4 req link`, `s4 req import` |
| **API** | CRUD requirements, bulk import |
| **UI** | Rich editor with kind templates, trace drag-and-drop |

### 6.2 Edit Operations

| Operation | Effect |
|-----------|--------|
| **Create** | New node; provenance `human`; confidence 1.0 |
| **Update statement** | Supersede old node; new revision ID |
| **Reparent** | Change decomposition tree |
| **Link trace** | Add/update TraceLink |
| **Waive trace** | Create WaiverRecord with reason + approver |
| **Activate / deprecate** | Status transition |

### 6.3 Validation Rules (on save)

1. `must` requirements require trace or waiver before `active`
2. AcceptanceCriterion should link to `verified_by` test or marked manual
3. ApiContract must reference valid spec artifact or inline schema
4. No circular `depends_on`
5. PerformanceConstraint must have parseable metric + unit

### 6.4 Audit

Every manual edit → `RequirementChangeEvent` (same pattern as SKG curation):

```
{ actor, timestamp, before_artifact, after_artifact, change_type }
```

---

## 7. Linking to Code (UCG)

### 7.1 Trace Strategies

| Strategy | Mechanism | Confidence |
|----------|-----------|------------|
| **Manual** | Human links requirement → symbol | 1.0 |
| **API contract** | OpenAPI operationId → UCG callable | 0.95 |
| **SKG bridge** | Requirement → Concept → `realizes` → symbol | 0.7–0.95 |
| **Heuristic** | Keyword match in symbol names / routes | 0.6–0.8 (proposed) |
| **LLM suggest** | Proposal only | ≤ 0.75 |
| **Coverage tool** | Existing traceability matrix import | 0.9 |

### 7.2 Architectural & Security Links

- **ArchitecturalConstraint** → UCG architectural view (`violates` / `conforms_to` on boundaries)
- **SecurityConstraint** → UCG symbols (auth middleware) + SKG Concept (Authentication)

### 7.3 Staleness Detection

When UCG snapshot advances:

```
for link in satisfies_edges(requirement):
  if ucg_node_removed(link.target):
    mark link stale; confidence *= 0.5; queue review
```

---

## 8. Linking to Tests

### 8.1 Test Reference Model

```
TestCase {
  id:           TestRef           // external: file:line, junit name, playwright id
  kind:         unit | integration | e2e | manual
  artifact:     Option<ArtifactId>  // test source file
  ucg_symbol:   Option<NodeId>      // test function symbol
}
```

**Edges:**
```
AcceptanceCriterion --[verified_by]--> TestCase
BusinessRequirement --[verified_by]--> TestCase  (via AC rollup)
```

### 8.2 Extraction Sources

| Source | Mapping |
|--------|---------|
| `@Requirement(BR-001)` annotations | Direct link (confidence 1.0) |
| Gherkin tags `@BR-001` | AC linkage |
| Test name conventions | Heuristic |
| Coverage reports (Jacoco, llvm-cov) | Indirect satisfaction evidence |

### 8.3 Rollup Rule

Parent requirement **satisfaction status** derived from children:

```
BR satisfied iff all must-children (AC, nested BR) have accepted verified_by
         OR explicit waiver on gap
```

---

## 9. Linking to Verification

### 9.1 Verification Integration

```
Requirement --[verified_by]--> VerificationRun {
  rule_set: string,
  passed: bool,
  artifact: ArtifactId,
  snapshot: SnapshotId
}
```

| Requirement kind | Verifier type |
|------------------|---------------|
| PerformanceConstraint | Metric threshold verifier |
| SecurityConstraint | Security scanner + policy engine |
| ArchitecturalConstraint | Architecture rule verifier |
| ApiContract | Contract test / schema diff |
| BusinessRequirement | Trace completeness verifier |

### 9.2 Trace Completeness Verifier

Certification gate:

```
forall req in active_requirements where priority == must:
  exists accepted trace (satisfies | verified_by) OR valid waiver
```

Emits `VerificationRun` artifact consumed by `s4-certification`.

---

## 10. Version History

### 10.1 Requirement Revision Chain

Requirements are **immutable once published**; edits create new revision:

```
RequirementRev-1 (active) ──supersedes──> RequirementRev-2 (active)
```

```
RequirementRevision {
  requirement_id: stable_id,      // BR-001 across revisions
  revision:       u32,
  effective_from: ISO-8601,
  sow_revision:   Option<string>, // "SOW v2.3 section 4.1"
  node:           RequirementNode snapshot
}
```

### 10.2 SOW Alignment

```
SowDocument {
  id: ArtifactId,
  version: string,
  parsed_snapshot: RequirementsSnapshotId,
  diff_from_previous: Option<RequirementsDiffArtifact>
}
```

When SOW v2 arrives:
1. Parse → proposed requirements
2. Diff against v1 graph (`added | changed | removed | unchanged`)
3. Human maps changes to supersession or new IDs
4. Trace links inherit forward when targets unchanged

### 10.3 Baselines

```
RequirementsBaseline {
  name: "Release 2.1",
  snapshot_id: RequirementsSnapshotId,
  code_snapshot: SnapshotId,
  skg_snapshot: KnowledgeSnapshotId,
  sealed_at: ISO-8601
}
```

Certification runs against **baseline triple** (requirements + code + knowledge).

---

## 11. Confidence

### 11.1 Confidence by Origin

| Origin | Initial confidence | Lifecycle |
|--------|-------------------|-----------|
| Human authored | 1.0 | accepted |
| Structured import (ReqIF, OpenAPI) | 0.95–1.0 | accepted |
| Template SOW parser | 0.85–0.95 | accepted or review |
| LLM SOW extraction | 0.5–0.75 | proposed |
| Heuristic code link | 0.6–0.8 | proposed |
| Human accepted link | 1.0 | accepted |

### 11.2 Trace Link Confidence

Independent from requirement text confidence:

```
link.confidence = f(extraction_method, source_agreement, staleness)
```

Multi-source agreement (manual + OpenAPI + test annotation) → boost toward 1.0.

### 11.3 Certification Query Filter

```
certification_view =
  requirements where status == active
  AND traces where lifecycle == accepted AND confidence >= 0.9
```

---

## 12. Interaction with the Software Knowledge Graph

### 12.1 Complementary Roles

| Dimension | Requirements Graph | Software Knowledge Graph |
|-----------|-------------------|-------------------------|
| **Origin** | Contractual (SOW) | Inferred + curated meaning |
| **Normativity** | SHALL/MUST obligations | Descriptive ("what Invoice means") |
| **Audience** | Legal, PM, compliance | Architects, domain experts |
| **Change driver** | SOW amendment | Code/doc evolution |
| **Volatility** | Versioned explicitly | Continuous refinement |

**Neither replaces the other.** A SOW says "shall process invoices"; SKG defines what Invoice *is* in this system.

### 12.2 Cross-Graph Edges

| Edge | Direction | Purpose |
|------|-----------|---------|
| `traces_to` | Requirement → Concept | "BR-001 implements Customer Management concept" |
| `belongs_to` | Requirement → Domain | Same domain model as SKG |
| `aligns_with` | BusinessRequirement → Capability (SKG) | Outcome ↔ capability |
| `constrains` | SecurityConstraint → Concept | SEC rule applies to PII concept |
| `refines` | Requirement → BusinessRule (SKG) | Detailed rule from contract clause |

### 12.3 Joint Workflows

**SOW onboarding:**
1. Parse SOW → Requirements Graph
2. Extractor proposes SKG Concepts from same SOW text
3. Human links `BR-* --traces_to--> Concept:*`
4. Code mapping: Concept `--realizes-->` UCG (SKG) + Requirement `--satisfies-->` UCG

**Impact analysis:**
```
Change BR-001 → find linked Concepts → find UCG symbols → find tests
```

**Gap detection:**
```
Concept in SKG with no Requirement traces_to → potential undocumented scope
Requirement with no Concept → orphan contract clause or missing domain model
```

### 12.4 Single Query Facade (Future)

`IntentQuery` spans both graphs:

```
"MFA requirement status"
  → SEC-003 (Requirements)
  → Authentication (SKG Concept)
  → AuthMiddleware.authenticate (UCG)
  → test_auth_mfa.rs (Test)
  → verification_run_2025-07-01 (passed)
```

---

## 13. Persistence

### 13.1 Requirements Snapshot Manifest

```
RequirementsSnapshotManifest {
  snapshot_id:       SnapshotId
  sow_document:      Option<ArtifactId>
  parent:            Option<SnapshotId>
  node_shards:       Vec<ArtifactId>    // by kind or hierarchy subtree
  trace_shards:      Vec<ArtifactId>    // all TraceLinks
  cross_skg_shards:  Vec<ArtifactId>
  cross_ucg_shards:  Vec<ArtifactId>
  change_log:        ArtifactId
  baseline_ref:      Option<BaselineId>
}
```

Stored in CAS via `s4-storage` — same immutability model as UCG and SKG.

### 13.2 Indexes

| Index | Purpose |
|-------|---------|
| `external_id` → RequirementId | JIRA, DOORS ID |
| `stable_id + revision` | History lookup |
| `kind + status` | Certification scans |
| `untraced_must` | Compliance queue |

---

## 14. Plugin Ecosystem

| Plugin role | Trait | Function |
|-------------|-------|----------|
| Document importer | `Importer` | SOW file → text artifact |
| Requirement parser | `RequirementParser` | Text → proposals |
| OpenAPI importer | `RequirementParser` | Spec → ApiContract |
| Trace suggester | `KnowledgeExtractor` | Propose code/test links |
| Trace verifier | `Verifier` | Completeness gates |

---

## 15. Phased Delivery

| Phase | Deliverable |
|-------|-------------|
| **R0** | Requirement kinds schema; TraceLink model |
| **R1** | Manual CRUD + trace to UCG |
| **R2** | OpenAPI → ApiContract import |
| **R3** | SOW Markdown parser + review UI |
| **R4** | Test annotation linking |
| **R5** | SKG cross-edges + joint queries |
| **R6** | SOW diff + revision chain |
| **R7** | Certification baseline triple |

---

## 16. Open Decisions

1. **ReqIF** as primary enterprise interchange vs custom YAML
2. **Waiver workflow** — single approver vs multi-party
3. **Requirement IDs** — human-assigned only vs auto + alias
4. **LLM auto-accept** for structured template SOWs with confidence > 0.9?
5. **Merge Requirements into SKG** as layer vs strict separation (current: **separate**)

---

## 17. Summary

The Requirements Graph:

1. **Turns SOW into machine-readable** typed requirements (BR, API, PERF, SEC, ARCH, AC)
2. **Makes every requirement traceable** — code, tests, verification, or explicit waiver
3. **Supports parse + manual edit** with full audit trail
4. **Versions with SOW revisions** — supersession chain and baselines
5. **Scores confidence** by origin; LLM and heuristics stay proposed until accepted
6. **Links to SKG** via `traces_to` / `aligns_with` — contract intent meets domain meaning
7. **Links to UCG and verification** — certification closes the loop from clause to evidence

Together with SKG and UCG, it completes the Digital Twin: **what was promised**, **what it means**, and **what was built**.
