# Software Knowledge Graph (SKG)
## Architecture Specification v0.1

> **Status:** Target spec.
> **Shipped:** naming-heuristic concept extract (`s4 knowledge extract`, satellite, always Proposed). The Digital Twin / full SKG is **not** shipped.
> **Product:** [Porting Workflow](../guides/PORTING_WORKFLOW.md)
> **Contract crate:** `s4-knowledge` (+ `s4-requirements` for trace edges)
> **Companion:** [Universal Code Graph](../graph/UNIVERSAL_CODE_GRAPH.md) (structure — not meaning)
> **Principle:** The SKG is the **Digital Twin** of what the software *means*, not what it *contains*.

---

## 1. Purpose

The **Software Knowledge Graph (SKG)** represents **meaning**: business concepts, domains, rules, ownership, capabilities, and processes. It answers questions the code graph cannot:

| Question | SKG | UCG (code graph) |
|----------|-----|------------------|
| What is an Invoice in this system? | ✅ Concept node + definition | ❌ `InvoiceService.java` |
| Who owns customer data? | ✅ DataOwnership edge | ❌ DB table name |
| What business rule governs checkout? | ✅ BusinessRule node | ❌ `if (cart.total > …)` |
| What code implements Authentication? | 🔗 Cross-edge `realizes` | ✅ Callable nodes |

**The SKG is the product.** The UCG is evidence. LLMs are extractors. Humans are curators.

---

## 2. Digital Twin — What It Means

A **Digital Twin** is a living, versioned, auditable model of the software system at the **semantic and intent layer**, continuously aligned with (but not identical to) the codebase.

```
                    ┌─────────────────────────┐
                    │   SOFTWARE KNOWLEDGE     │
                    │   GRAPH (meaning)        │
                    │                          │
                    │  Invoice · Customer ·    │
                    │  Checkout · Auth · Rules │
                    └───────────┬─────────────┘
                                │ cross-edges
                    ┌───────────▼─────────────┐
                    │   UNIVERSAL CODE GRAPH   │
                    │   (structure)            │
                    │                          │
                    │  modules · symbols ·     │
                    │  calls · dependencies    │
                    └───────────┬─────────────┘
                                │ evidence
                    ┌───────────▼─────────────┐
                    │   Repository / runtime   │
                    └─────────────────────────┘
```

| Twin property | SKG mechanism |
|---------------|---------------|
| **Faithful** | Trace edges to code; confidence scores; verification |
| **Current** | Incremental extraction on commit; snapshot chain |
| **Explainable** | Provenance on every fact; human + LLM attribution |
| **Correctable** | Human curation workflow; never silent overwrite |
| **Historical** | Snapshot time-travel — "what did we believe at release X?" |
| **Actionable** | Drives requirements trace, certification, refactor planning |

**Why "twin" not "documentation":** Documentation drifts. The SKG is **machine-readable**, **linked to code**, **versioned with snapshots**, and **governed by acceptance workflow** — it behaves like an operational model organizations can certify against.

---

## 3. SKG vs UCG — Strict Separation

| Aspect | SKG | UCG |
|--------|-----|-----|
| **Nodes** | Concepts, domains, rules, capabilities | Files, symbols, calls |
| **Language** | Business + domain vocabulary | Programming language neutral syntax/semantics |
| **Truth source** | Multi-source inference + human acceptance | Parse + link (deterministic) |
| **LLM role** | Primary extractor (proposed) | Context only; optional summarization |
| **Storage** | Knowledge snapshot artifacts | Graph shard artifacts |
| **Crate** | `s4-knowledge` | `s4-graph` |

**Cross-graph edges** (`realizes`, `implements_concept`, `traces_to`, `violates_rule`) are **first-class SKG facts** stored with provenance — never implicit joins in application code.

---

## 4. Node Taxonomy (Meaning Layer)

### 4.1 Core Node Kinds

| Kind | Purpose | Examples |
|------|---------|----------|
| **Concept** | Named business or domain idea | Invoice, Customer, Authentication, Order |
| **Domain** | Bounded context grouping concepts | Commerce, Identity, Reporting, Billing |
| **Capability** | What the system can do for users/stakeholders | Process Payments, Generate Reports, Authenticate Users |
| **Process** | End-to-end business flow (not code flow) | Checkout, Order Fulfillment, Monthly Reporting |
| **BusinessRule** | Constraint or policy the system enforces | "Invoice total must equal line items sum" |
| **DataOwnership** | Stewardship of a concept's data | Customer PII → Legal/Compliance owner |
| **Policy** | Non-functional or governance rule | Retention period, GDPR scope |
| **GlossaryTerm** | Canonical definition in enterprise vocabulary | Synonyms link here |
| **Assumption** | Explicit belief about the system | "Orders are never deleted, only archived" |
| **Risk** | Identified semantic/architectural risk | "Duplicate Invoice concept in two domains" |

### 4.2 Example Instantiation

```
Domain: Commerce
  ├── Concept: Order
  ├── Concept: Invoice
  ├── Concept: Checkout
  ├── Process: CheckoutFlow (Order → Payment → Confirmation)
  ├── BusinessRule: invoice-total-integrity
  └── Capability: ProcessCheckout

Domain: Identity
  ├── Concept: Authentication
  ├── BusinessRule: mfa-required-for-admin
  └── DataOwnership: credentials → Security team

Domain: Reporting
  ├── Concept: Reporting
  ├── Capability: GenerateFinancialReports
  └── Process: MonthlyCloseReporting
```

### 4.3 Node Envelope

Same meta-model as canonical data model:

```
KnowledgeNode {
  id:           KnowledgeNodeId
  kind:         KnowledgeKind
  label:        string              // primary name
  description:  string              // human-readable meaning
  aliases:      string[]            // Order, SalesOrder, purchase
  domain_id:    Option<DomainId>
  envelope:     EntityEnvelope      // provenance, confidence, lifecycle
  extensions:   map
}
```

**Why aliases:** Code uses `SalesOrder`, docs say `Order`, DB says `orders` — one Concept, many names.

---

## 5. Edge Taxonomy (Semantic Relationships)

| Kind | From → To | Meaning |
|------|-----------|---------|
| `belongs_to` | Concept → Domain | Bounded context membership |
| `related_to` | Concept → Concept | Weak association |
| `part_of` | Concept → Concept | Composition (Invoice part_of Order) |
| `precedes` | Concept/Process → Concept/Process | Temporal/business ordering |
| `defines` | GlossaryTerm → Concept | Canonical definition |
| `synonym_of` | Concept → Concept | Equivalent meaning (pending merge) |
| `implements` | Capability → Process | Capability realized by process |
| `constrains` | BusinessRule → Concept/Process | Rule applies to |
| `owns_data` | DataOwnership → Concept | Stewardship |
| `depends_on` | Domain → Domain | Context map dependency |
| `conflicts_with` | Concept → Concept | Detected duplication/ambiguity |
| **Cross to UCG** | | |
| `realizes` | Concept/Capability → UCG SymbolId | Code implements meaning |
| `evidenced_by` | KnowledgeNode → ArtifactId | Doc, ticket, config file |
| `traces_to` | Requirement → Concept | Intent link |
| `violates` | UCG Finding → BusinessRule | Compliance gap |

**Why separate semantic edges from UCG `calls`/`depends_on`:** Business "Checkout precedes Invoice" is not a function call. Mixing layers corrupts both graphs.

---

## 6. Concept Extraction — Pipeline Design

Extraction is **multi-source**, **multi-stage**, and **never single-authority**.

```
┌─────────────────────────────────────────────────────────────────┐
│                    EXTRACTION SOURCES                            │
├──────────┬──────────┬──────────┬──────────┬──────────────────┤
│   Code   │   Docs   │  Reqs    │  Config  │  Annotations/API   │
│  (UCG)   │ (import) │ (SKG)    │ (import) │  (OpenAPI, DB)     │
└────┬─────┴────┬─────┴────┬─────┴────┬─────┴─────────┬──────────┘
     │          │          │          │               │
     ▼          ▼          ▼          ▼               ▼
┌─────────────────────────────────────────────────────────────────┐
│              DETERMINISTIC EXTRACTORS (plugins)                  │
│  naming heuristics · route patterns · schema reflection ·        │
│  requirement parsers · glossary import                           │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│              HEURISTIC ANALYZERS                                 │
│  co-occurrence · module boundary clustering · duplicate detect  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│              LLM EXTRACTORS (proposed only)                      │
│  concept induction · rule extraction · ownership inference       │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│              KNOWLEDGE FUSION                                    │
│  merge candidates · confidence aggregation · conflict detection  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│              HUMAN CURATION QUEUE                                │
│  accept · reject · merge · edit · assign domain                  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
                    SKG Snapshot (accepted facts)
```

### 6.1 Extractor Roles (Plugins)

| Extractor | Trait | Input | Output |
|-----------|-------|-------|--------|
| **CodeConceptExtractor** | `KnowledgeExtractor` | UCG slice | Concept candidates + `realizes` edges |
| **DocConceptExtractor** | `KnowledgeExtractor` | Doc artifacts | Concepts + `evidenced_by` |
| **SchemaExtractor** | `KnowledgeExtractor` | SQL/Proto/OpenAPI | Concepts from entities/endpoints |
| **RequirementLinker** | `KnowledgeExtractor` | Requirements graph | `traces_to` edges |
| **RuleExtractor** | `KnowledgeExtractor` | UCG + docs | BusinessRule candidates |
| **OwnershipInferencer** | `KnowledgeExtractor` | Docs + config + LLM | DataOwnership candidates |
| **LlmConceptExtractor** | `LlmProvider` | Context bundle | Proposal artifact |

**Core never implements extractors** — only `KnowledgeExtractor` trait and fusion rules.

### 6.2 Code-Driven Extraction (Deterministic)

Heuristics over UCG (high confidence when matched):

| Signal | Example | Inferred concept |
|--------|---------|------------------|
| Class/module name | `InvoiceService`, `invoice.rs` | Concept: Invoice |
| API route | `POST /orders/{id}/checkout` | Process: Checkout |
| DB table/model | `customers` table | Concept: Customer |
| Event name | `OrderCreated` | Concept: Order |
| Package/domain path | `com.acme.billing.*` | Domain: Billing |
| Test description | `"should authenticate admin users"` | Concept: Authentication |

**Confidence:** 0.75–0.95 depending on signal strength (exact name match vs substring).

### 6.3 Documentation & Requirements Extraction

- Imported Markdown, Confluence, PDF (text artifacts) → NLP/LLM chunk → concept mentions
- Explicit glossary files → `GlossaryTerm` + `defines` (confidence 1.0 if human-authored source marked trusted)
- Requirements text → Concept linkage via `traces_to`

### 6.4 LLM Extraction (Proposed)

LLM receives **bounded context** (UCG slice + doc chunks + existing SKG neighborhood):

```
ReasonIntent: ExtractConcepts | ExtractRules | InferOwnership | MapDomain
Output: KnowledgeProposal artifact (never direct SKG write)
```

Extracted items:
- New Concept candidates with descriptions
- BusinessRule natural language statements
- DataOwnership assignments
- Domain boundary suggestions
- `synonym_of` / `conflicts_with` flags

**All LLM outputs:** `lifecycle: proposed`, `confidence: model-assigned (capped ≤ 0.85)`.

---

## 7. Confidence Scores

### 7.1 Confidence Model

`Confidence` is a float `0.0–1.0` on every SKG fact, independent of lifecycle.

| Band | Meaning |
|------|---------|
| **1.0** | Human accepted or authoritative source (signed glossary) |
| **0.9–0.99** | Deterministic extractor, multiple agreeing sources |
| **0.7–0.89** | Strong heuristic single source |
| **0.5–0.69** | LLM or weak heuristic |
| **< 0.5** | Speculative — never auto-accept |

### 7.2 Aggregation Formula (Fusion)

When multiple extractors propose the same Concept (aligned by label similarity + embedding optional):

```
aggregated_confidence = 1 - ∏(1 - c_i)   // independent source combination
cap at 0.95 unless human accepted → 1.0
```

**Conflict penalty:** If `conflicts_with` detected between merges, multiply by 0.5 until human resolves.

### 7.3 Confidence Decay (Optional)

For accepted facts tied to code mappings:

```
if code_realizes_edge_stale(snapshot_base, snapshot_head):
  confidence *= 0.9   // mapping may be outdated; queue re-verification
```

**Why decay:** Twin drifts when code refactors but SKG mapping unchanged. Triggers curation queue, not automatic deletion.

### 7.4 Confidence in Queries

Default query filter: `confidence >= 0.7 AND lifecycle == accepted`.

Certification mode: `confidence == 1.0 AND provenance.source_type != reasoner`.

Exploration mode: include `proposed` for human review UI.

---

## 8. LLM Interaction Design

### 8.1 Principles

| Principle | Implementation |
|-----------|----------------|
| **LLM is not truth** | All outputs are `KnowledgeProposal` artifacts |
| **Provider agnostic** | `LlmProvider` plugin; `s4-llm` contracts |
| **Bounded context** | `ContextSlicer` from UCG + SKG + docs |
| **Structured output** | JSON schema enforced proposals |
| **Reproducibility** | `ModelMetadata` with prompt/response hashes |
| **No silent merge** | Fusion engine queues; human or verifier accepts |

### 8.2 Interaction Modes

| Mode | Trigger | LLM task |
|------|---------|----------|
| **Induction** | New snapshot / major diff | Discover concepts from changed code+docs |
| **Enrichment** | Human selects Concept | Generate description, aliases, related concepts |
| **Rule extraction** | Analyzer flags validation logic | Propose BusinessRule text |
| **Ownership inference** | Compliance scan | Propose DataOwnership from doc patterns |
| **Conflict resolution** | Duplicate concepts detected | Suggest merge or distinguish |
| **Explanation** | User query | Natural language over SKG (read-only) |

### 8.3 Proposal Artifact

```
KnowledgeProposal {
  kind:           concept_batch | rule_batch | ownership_batch | merge_suggestion
  claims:         KnowledgeClaim[]
  rationale:      ArtifactId
  model:          ModelMetadata
  lifecycle:      proposed          // always
}

KnowledgeClaim {
  action:         create | update | link | merge
  node:           KnowledgeNode draft
  edges:          KnowledgeEdge draft[]
  confidence:     f32               // model self-score, capped
}
```

### 8.4 Human-in-the-Loop Gates

| Gate | Rule |
|------|------|
| Auto-accept | **Never** for LLM-only claims |
| Auto-accept | Deterministic extractor + existing accepted Concept exact match → link only |
| Review required | New Concept, BusinessRule, DataOwnership, merge |
| Verifier optional | Batch accept after conformance rules pass |

---

## 9. Human Corrections

### 9.1 Curation Workflow

```
CurationItem {
  proposal_id:    ArtifactId
  item_type:      concept | rule | edge | merge
  status:         pending | accepted | rejected | deferred
  reviewer:       UserId
  timestamp:      ISO-8601
  comment:        optional
}
```

**Actions:**

| Action | Effect |
|--------|--------|
| **Accept** | `lifecycle → accepted`, `confidence → 1.0`, provenance `source_type: human` |
| **Reject** | `lifecycle → rejected`; proposal archived |
| **Edit & accept** | Modified payload; provenance records human edit diff |
| **Merge** | Combine Concepts; `synonym_of` → redirect; reattach edges |
| **Split** | One Concept → two; human defines boundary |
| **Assign domain** | `belongs_to` edge created/updated |
| **Adjust mapping** | Change `realizes` target symbol; confidence → 1.0 |
| **Override confidence** | Human sets explicit confidence with reason |

### 9.2 Audit Trail

Every correction produces:

```
CurationEvent {
  event_id, snapshot_id, actor, action,
  before: ArtifactId?,   // previous state blob
  after:  ArtifactId,     // new state blob
}
```

**Why:** Certification requires knowing *who* accepted *what* and *when*. Digital Twin governance is legal-grade, not wiki-grade.

### 9.3 UI Surfaces (`s4-ui`)

- **Review queue** — proposed items sorted by impact (new Concept > edge update)
- **Twin explorer** — domain → concept → code mappings → rules
- **Diff view** — SKG snapshot diff alongside code diff
- **Conflict resolver** — side-by-side merge UI

---

## 10. Persistence

### 10.1 Storage Model

SKG persists as **immutable knowledge snapshots** in CAS — parallel to UCG snapshots, linked by manifest.

```
KnowledgeSnapshotManifest {
  snapshot_id:        SnapshotId          // aligns with or references code snapshot
  code_snapshot:      SnapshotId          // evidence anchor
  node_shards:        Vec<ArtifactId>     // sharded by domain hash
  edge_shards:        Vec<ArtifactId>
  cross_graph_shards: Vec<ArtifactId>     // realizes, evidenced_by to UCG
  index_shards:       Vec<ArtifactId>     // label, alias, concept_id lookup
  curation_log:       ArtifactId          // append-only events
  stats:              { concept_count, rule_count, proposed_pending }
}
```

### 10.2 Shard Strategy

| Shard key | Rationale |
|-----------|-----------|
| `domain_id` | Queries are domain-scoped ("show Commerce twin") |
| `knowledge_kind` | Rules vs concepts separated for cert scans |

Same CSR/columnar patterns as UCG where applicable — SKG is smaller but same infrastructure.

### 10.3 Cross-Graph Persistence

`realizes` edges store:

```
CrossGraphEdge {
  skg_node:     KnowledgeNodeId
  ucg_node:     NodeId              // includes snapshot scope
  confidence:   f32
  lifecycle:    FactLifecycle
  provenance:   Provenance
}
```

Stored in dedicated cross-graph shard — enables "all code for Invoice concept" without duplicating UCG nodes.

### 10.4 Proposed vs Accepted Storage

**Option (recommended):** Separate overlay shard for `proposed` facts; merge into accepted shard on curation. Queries choose layer.

**Why not delete proposals on reject:** Audit and model improvement need negative examples.

---

## 11. Incremental Updates

On code snapshot `S+1`:

1. Run deterministic extractors on UCG delta
2. Propose new/updated mappings
3. Mark stale `realizes` edges (symbol removed) → curation queue
4. LLM induction on changed modules only (budgeted)
5. Fusion → proposed overlay
6. Publish `KnowledgeSnapshot S+1` when accepted items merged (or auto-link-only applied)

**Twin sync lag:** SKG may trail code by review queue depth — explicit `twin_status: synced | pending_review | stale` on manifest.

---

## 12. Query Patterns (Meaning-First)

| Query | SKG traversal |
|-------|---------------|
| "What is Customer?" | Concept lookup → description + aliases + domain |
| "What implements Checkout?" | Concept → `realizes` → UCG symbols |
| "What rules apply to Invoice?" | Concept ← `constrains` — BusinessRule |
| "Who owns customer data?" | Concept ← `owns_data` — DataOwnership |
| "Impact of changing Authentication?" | Concept → `realizes` → UCG → callers (cross to UCG) |
| "Untrusted knowledge?" | `lifecycle: proposed OR confidence < 0.7` |

---

## 13. Integration Map

| Component | Role |
|-----------|------|
| `s4-knowledge` | SKG types, fusion traits, curation API |
| `s4-graph` / UCG | Evidence source for extractors |
| `s4-requirements` | Requirements ↔ Concept traces |
| `s4-llm` | Proposal types |
| `s4-verification` | Rule compliance checks |
| `s4-certification` | Twin certification at snapshot |
| `s4-plugin` | `KnowledgeExtractor` plugin role |
| `s4-storage` | CAS artifacts |
| `s4-ui` | Curation queue, twin explorer |

---

## 14. Digital Twin Lifecycle

```
Onboard project
  → Import code + docs + requirements
  → Bootstrap SKG (extract + heavy LLM induction)
  → Human curation sprint (accept core concepts/domains)
  → Baseline certified twin (Snapshot T0)

Each commit/CI
  → UCG delta
  → Incremental extraction
  → Review queue (if material)
  → Twin sync status updated

Release
  → Freeze KnowledgeSnapshot + CodeSnapshot pair
  → Certification against BusinessRules + Requirements
  → Archive as auditable twin revision

Refactor / reorg
  → Code mappings drift
  → Confidence decay + re-mapping proposals
  → Human confirms twin still valid
```

**Outcome:** Stakeholders interact with **Invoice, Customer, Checkout** — not file trees. Engineers trace to code on demand. Compliance certifies against **rules and ownership**, not line counts.

---

## 15. Phased Delivery

| Phase | Deliverable |
|-------|-------------|
| **K0** | SKG node/edge schema; KnowledgeSnapshot manifest |
| **K1** | Code-driven deterministic extractor (naming) |
| **K2** | Fusion + confidence; proposed overlay |
| **K3** | Human curation API + audit log |
| **K4** | LLM induction proposals |
| **K5** | BusinessRule + DataOwnership extractors |
| **K6** | Twin certification bundle (SKG + UCG pair) |
| **K7** | Confidence decay + stale mapping detection |

---

## 16. Open Decisions

1. **Embedding-based concept dedup** — local model vs LLM-only matching
2. **SKG size** — separate DB vs always CAS shards
3. **Auto-accept thresholds** for deterministic extractors when Concept already exists
4. **External glossary sync** — ServiceNow, LeanIX, enterprise CMDB
5. **Twin visualization standard** — C4 domain model export from SKG?

---

## 17. Summary

The Software Knowledge Graph:

1. **Represents meaning** — Concepts, Domains, Rules, Ownership, Capabilities, Processes
2. **Is not the code graph** — linked via explicit cross-edges, never merged
3. **Extracts from many sources** — code heuristics, docs, requirements, schemas, LLM
4. **Scores confidence** — multi-source fusion; LLM capped; human sets truth to 1.0
5. **Governs LLM output** — proposals only; structured artifacts; full provenance
6. **Empowers human correction** — accept, reject, merge, split, audit trail
7. **Persists as versioned snapshots** — parallel to code; CAS-backed; overlay for proposed
8. **Is the Digital Twin** — faithful, current, explainable, correctable, historical, certifiable model of what the system *means*

The codebase is the body. The SKG is the mind. S4MP keeps them linked — and keeps both honest.
