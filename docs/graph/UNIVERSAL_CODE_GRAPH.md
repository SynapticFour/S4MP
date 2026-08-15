# Universal Code Graph (UCG)
## Architecture Specification v0.1

> **Status:** Target spec.
> **Shipped:** in-memory semantic graph + filter query in `s4-graph` (used by `s4 graph` / `s4 query`). Distributed store, S4QL, and `s4-graph-engine` are **not** shipped.
> **Product:** [Porting Workflow](../guides/PORTING_WORKFLOW.md)
> **Crate home:** `s4-graph`
> **Depends on:** [Canonical Data Model](../model/CANONICAL_MODEL.md)

---

## 1. Purpose

The **Universal Code Graph (UCG)** is the central, language-independent representation of every software project analyzed by S4MP. It materializes the canonical data model as a **typed, layered, versioned multigraph** that all parsers, analyzers, reasoners, and surfaces query through a single API.

| Principle | Decision |
|-----------|----------|
| Language | Nodes are software **concepts**, not syntax constructs |
| Identity | `Symbol` is the semantic anchor; physical nodes are separate |
| Mutability | **Immutable snapshots** + append-only deltas |
| Scale | Sharded, columnar, lazily loaded — not one in-memory `HashMap` |
| Truth | Edges and nodes carry provenance, confidence, lifecycle |
| Query | Traversal-first API; declarative query layer above it |

**Why a dedicated graph architecture:** Code is inherently relational. Tables flatten away call paths, dependency direction, and cross-layer traceability. The UCG preserves structure while remaining serializable, diffable, and certifiable.

---

## 2. Position in the Platform

```
Import → Parse → Link ──► UCG Materialization ──► Projections
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
         s4-metrics    s4-analysis    s4-requirements
              │               │               │
              └───────────────┴───────────────┘
                              ▼
                    s4-llm (context slices)
                              ▼
                    s4-verification / s4-certification
```

The UCG is **not** the Software Knowledge Graph in full — it is the **code-centric subgraph**. Knowledge entities (`Concept`, `Domain`, `Requirement`) attach to UCG nodes via cross-layer edges but may live in companion stores (`s4-knowledge`, `s4-requirements`).

---

## 3. Graph Model

### 3.1 Multigraph Properties

| Property | Value |
|----------|-------|
| **Directed** | Yes (default); undirected views derived |
| **Multigraph** | Yes — multiple edge kinds between same pair allowed |
| **Hyperedges** | No in v1; n-ary relations modeled as stub nodes |
| **Self-loops** | Allowed (recursive calls) |
| **Layers** | Logical partitions over same node ID space |
| **Weighted** | Optional `confidence` on every edge |

### 3.2 Node vs Concept

| Term | Meaning |
|------|---------|
| **Node** | Stored graph vertex with ID, kind, payload |
| **Concept** | Canonical model entity; may map 1:N to nodes across snapshots |
| **Projection** | Layer-filtered read-only view over stored nodes/edges |

---

## 4. Node Taxonomy

Nodes are grouped into ** tiers**. Tier determines storage shard, index strategy, and typical cardinality.

### 4.1 Tier P — Physical (high cardinality, low semantics)

| Kind | Canonical entity | Payload highlights | Typical count |
|------|------------------|-------------------|---------------|
| `repository` | Repository | uri, vcs_kind, revision | 1–50 per project |
| `file` | File | path, content_hash, language_id | 10³–10⁵ |
| `directory` | (supporting) | path prefix | 10²–10⁴ |

**Why:** Anchors all semantic nodes to locatable bytes. Never merged with semantic identity.

### 4.2 Tier S — Structural (namespace topology)

| Kind | Canonical entity | Payload highlights | Typical count |
|------|------------------|-------------------|---------------|
| `module` | Module | qualified_name, visibility | 10²–10⁴ |
| `package` | Module (top-level) | package coordinates, version | 10¹–10³ |
| `external_unit` | External dependency | package id, version range | 10²–10⁴ |

**Why:** Dependency and architecture analysis operate at module granularity before symbol granularity.

### 4.3 Tier M — Semantic (core identity)

| Kind | Canonical entity | Payload highlights | Typical count |
|------|------------------|-------------------|---------------|
| `symbol` | Symbol | qualified_name, role hint | 10⁴–10⁶ |
| `callable` | Callable | signature ref, parameters | 10⁴–10⁶ |
| `type_definition` | TypeDefinition | type_kind, members ref | 10³–10⁵ |
| `contract` | Contract | contract_kind, required surface | 10³–10⁵ |
| `value` | Symbol (const/static) | type ref, mutability | 10³–10⁵ |
| `macro` | Symbol (macro) | hygiene, expansion ref | 10²–10⁴ |

**Why:** `symbol` is the **identity hub**. Callable, type_definition, and contract are **typed facades** — same underlying symbol ID with role-specific payload shards.

**Design rule:** Analyzers reference `symbol` or role node interchangeably via `alias_of` edge.

### 4.4 Tier A — Analytical (derived, recomputed)

| Kind | Canonical entity | Payload highlights | Typical count |
|------|------------------|-------------------|---------------|
| `metric_node` | Metric (aggregate) | metric_kind, value | 10⁴–10⁶ |
| `finding` | (analysis) | severity, message | 10²–10⁵ |
| `boundary` | Architecture | member module refs | 10¹–10³ |
| `layer_tag` | Architecture | layer name, ordering | 10¹–10² |
| `pattern_instance` | Architecture | pattern kind, confidence | 10²–10³ |
| `flow_summary` | Flow | step count, entry ref | 10²–10⁴ |

**Why:** Derived nodes are **cache materializations** — disposable and rebuildable from semantic tier + rules.

### 4.5 Tier X — Extension

| Kind | Format | Rule |
|------|--------|------|
| `extension:*` | `ExtensionKindId` namespaced | Must register in ontology; stored opaquely if unknown |

### 4.6 Node Envelope (Storage)

Every node is stored as:

```
GraphNode {
  id:           NodeId           // { snapshot_id, local_id: u64 }
  kind:         NodeKind         // enum + extension
  tier:         NodeTier         // P | S | M | A | X — denormalized for sharding
  symbol_key:   Option<SymbolKey> // cross-snapshot identity when resolved
  payload_ref:  ArtifactId | inline bytes
  envelope:     EntityEnvelope   // provenance, lifecycle, confidence, schema_version
}
```

**Why tier denormalization:** Shard routing without parsing payload at read time.

---

## 5. Edge Taxonomy

Edges are **typed and directed**. Standard kinds are frozen slowly; extensions use namespaced IDs.

### 5.1 Containment & Structure

| Kind | From → To | Meaning |
|------|-----------|---------|
| `contains` | repository → file, module → symbol | Physical/logical containment |
| `defines` | file → module, module → symbol | Definition site |
| `declares` | module → symbol | Forward declaration / export |
| `part_of` | symbol → module | Membership (inverse index) |

### 5.2 Semantic Reference

| Kind | From → To | Meaning |
|------|-----------|---------|
| `references` | symbol → symbol | Unresolved or resolved name use |
| `aliases` | symbol → symbol | Same entity (typedef, re-export) |
| `specializes` | type_definition → type_definition | Inheritance / subtype |
| `implements` | type_definition → contract | Trait/interface impl |
| `signature_of` | callable → type_definition | Return/parameter types |

### 5.3 Behavioral

| Kind | From → To | Meaning |
|------|-----------|---------|
| `calls` | callable → callable | Invocation (static/dynamic/unknown) |
| `reads` | callable → symbol | Data read |
| `writes` | callable → symbol | Data write |
| `throws` | callable → type_definition | Exception/effect |

**Edge payload for `calls`:**
```
{ resolution: static|dynamic|unknown, site: SourceLocation, confidence: f32 }
```

### 5.4 Coupling

| Kind | From → To | Meaning |
|------|-----------|---------|
| `depends_on` | module → module | Compile/build dependency |
| `imports` | module → module | Namespace import (synonym cluster) |
| `uses_external` | module → external_unit | Third-party package |

**Why separate `calls` and `depends_on`:** Compile-time coupling exists without runtime calls (DI, interfaces, configs).

### 5.5 Cross-Layer (Knowledge attachment)

| Kind | From → To | Meaning |
|------|-----------|---------|
| `maps_to` | concept → symbol | Knowledge mapping |
| `realizes` | feature → callable | Feature entry point |
| `traces_to` | requirement → symbol | Traceability |
| `measures` | metric_node → symbol | Metric attachment |
| `annotates` | * → * | Generic annotation with role string |

### 5.6 Identity & Versioning

| Kind | From → To | Meaning |
|------|-----------|---------|
| `same_as` | symbol → symbol | Cross-snapshot identity link |
| `supersedes` | node → node | Version chain within lineage |
| `derived_from` | node → node | Derivation provenance |

### 5.7 Edge Envelope

```
GraphEdge {
  id:           EdgeId           // { snapshot_id, local_id: u64 }
  kind:         EdgeKind
  from:         NodeId
  to:           NodeId
  layer:        GraphLayer       // which projection owns this edge
  payload:      optional         // kind-specific (site, weight, etc.)
  envelope:     EntityEnvelope
}
```

---

## 6. Graph Layers (Projections)

Layers are **filters**, not separate graphs.

| Layer | Node kinds | Edge kinds | Produced by |
|-------|------------|------------|-------------|
| **Physical** | repository, file, directory | contains | Import |
| **Syntax** | (optional AST stubs) | child_of, covers | Parser |
| **Semantic** | symbol, callable, type_definition, contract | defines, references, calls, implements | Parse + Link |
| **Structural** | module, package, external_unit | depends_on, imports | Link |
| **Architectural** | boundary, layer_tag, pattern_instance | annotates, violates | Analysis |
| **Feature** | flow_summary | realizes, composed_of | Analysis |
| **Quality** | metric_node, finding | measures | Analysis |

**Why layers:** IDE wants semantic traversal; architect wants module dependencies; certifier wants requirement traces — one store, many projections.

---

## 7. Traversal API

Three API levels, outer depends on inner.

### 7.1 Level 0 — Storage Iterator (engine internal)

```
trait NodeStore {
  fn get(node_id: NodeId) -> Option<GraphNode>
  fn iter_by_kind(kind, shard) -> impl Iterator<Item = GraphNode>
  fn iter_by_symbol_key(key) -> impl Iterator<Item = NodeId>
}

trait EdgeStore {
  fn out_edges(from: NodeId, kind_filter: EdgeKindSet) -> impl Iterator<Item = GraphEdge>
  fn in_edges(to: NodeId, kind_filter: EdgeKindSet) -> impl Iterator<Item = GraphEdge>
}
```

### 7.2 Level 1 — Traversal API (public, `s4-graph`)

```
trait GraphTraversal {
  /// Single-hop neighborhood.
  fn neighbors(node: NodeId, spec: NeighborSpec) -> Result<NeighborSet>

  /// BFS/DFS with kind filters and depth limit.
  fn walk(start: NodeId, spec: WalkSpec) -> Result<WalkIterator>

  /// Shortest path under allowed edge kinds.
  fn path(from: NodeId, to: NodeId, spec: PathSpec) -> Result<Option<Path>>

  /// Reachability check without materializing path.
  fn reachable(from: NodeId, to: NodeId, spec: ReachSpec) -> Result<bool>
}
```

**NeighborSpec:**
```
{
  direction:    outgoing | incoming | both
  edge_kinds:   EdgeKindSet | All
  layer:        Option<GraphLayer>
  lifecycle:    accepted_only | include_proposed
  max_degree:   Option<u32>     // safety cap
}
```

**WalkSpec:**
```
{
  strategy:     bfs | dfs
  max_depth:    u32
  max_nodes:    u32             // budget — critical at scale
  edge_kinds:   EdgeKindSet
  node_filter:  Option<NodeKindSet>
  visit_fn:     optional callback for early termination
}
```

### 7.3 Level 2 — Declarative Query (S4QL, future `s4-query`)

Graph pattern matching above traversal:

```
MATCH (c:callable)-[:calls*1..3]->(t:callable)
WHERE c.module IN boundaries("domain/core")
RETURN t, path
```

**Why three levels:** Storage iterators are shard-aware; traversal encodes safety budgets; S4QL serves humans and LLM context bundlers.

### 7.4 Context Bundling (for `s4-llm`)

```
trait ContextSlicer {
  /// Extract bounded subgraph for reasoning.
  fn slice(seed: NodeId, budget: SliceBudget) -> Result<GraphSlice>
}

SliceBudget { max_nodes, max_edges, max_depth, prefer_kinds }
```

**Why:** LLM context windows require **deliberate subgraph extraction**, not whole-graph serialization.

---

## 8. Storage Format

### 8.1 Design Goals

1. **Immutable snapshot segments** — append-only, content-addressed
2. **Shard by tier + hash** — parallel I/O, bounded memory
3. **Columnar edges** — compress adjacency; cache-friendly scans
4. **O(1) node lookup** — via dense local ID → offset index
5. **Portable** — snapshot = manifest of artifact IDs in `s4-storage`

### 8.2 Snapshot Layout

```
SnapshotManifest {
  snapshot_id:     SnapshotId
  parent:          Option<SnapshotId>
  node_shards:     Vec<ArtifactId>    // one per (tier, shard_key)
  edge_shards:     Vec<ArtifactId>    // one per (layer, shard_key)
  index_shards:    Vec<ArtifactId>    // symbol_key, path, kind bitmaps
  stats:           { node_count, edge_count, tier_counts }
}
```

### 8.3 Node Shard Format

**Recommended: columnar record batch per shard**

| Column | Type | Purpose |
|--------|------|---------|
| `local_id` | u64 | dense ID within shard |
| `kind` | u16 | NodeKind discriminant |
| `symbol_key_hash` | u64 | optional, for cross-ref |
| `payload_offset` | u64 | into payload blob section |
| `lifecycle` | u8 | denormalized filter |
| `confidence` | f32 | denormalized filter |

Payloads stored in contiguous blob section (better compression).

**Serialization codec:** `postcard` or `rkyv` for zero-copy read; JSON for debug exports only.

### 8.4 Edge Shard Format — CSR (Compressed Sparse Row)

Edges stored in **CSR** per `(layer, shard)`:

```
row_ptr:   Vec<u64>           // len = node_count + 1
col:       Vec<u64>           // target local_ids
kind:      Vec<u16>           // EdgeKind per edge
payload:   Blob               // optional edge attributes
```

**Why CSR:** Industry standard for graph analytics; O(out_degree) neighbor access; excellent compression; maps well to mmap.

For **reverse edges** (predecessors): maintain transposed CSR or `roaring` bitmap inverted index per hot kinds.

### 8.5 Secondary Indexes (separate artifacts)

| Index | Key → Value | Purpose |
|-------|-------------|---------|
| `symbol_key_index` | SymbolKey → NodeId[] | Cross-snapshot lookup |
| `path_index` | file path → NodeId | IDE navigation |
| `kind_bitmap` | NodeKind → RoaringBitmap | Fast kind scans |
| `module_range` | module NodeId → symbol id range | Scoped queries |

**Why separate indexes:** Rebuild indexes without rewriting node/edge shards; swap index algorithms independently.

### 8.6 Storage Backend

| Tier | Backend | Rationale |
|------|---------|-----------|
| **Hot snapshot** | mmap'd shard files + in-memory index cache | Read-heavy IDE/query |
| **Cold snapshot** | CAS blobs in `s4-storage` (filesystem/S3) | Immutable archive |
| **Write path** | Build in temp dir → seal → register artifacts | Atomic snapshot publish |

**Recommended Rust crates:**

| Crate | Role |
|-------|------|
| [`memmap2`](https://crates.io/crates/memmap2) | Zero-copy shard access |
| [`postcard`](https://crates.io/crates/postcard) | Compact, deterministic serialize |
| [`rkyv`](https://crates.io/crates/rkyv) | Zero-copy archived views (if benchmarks justify complexity) |
| [`roaring`](https://crates.io/crates/roaring) | Bitmap indexes for kind/sets |
| [`zstd`](https://crates.io/crates/zstd) | Shard compression (cold storage) |
| [`blake3`](https://crates.io/crates/blake3) | Content addressing (already in `s4-core`) |

Avoid embedding a full graph DB (Neo4j) in v1 — the UCG format is **custom over CAS** for reproducibility and audit.

---

## 9. Incremental Updates

Snapshots are **immutable**. Incremental work produces **delta artifacts** merged at publish time.

### 9.1 Invalidation Unit

| Unit | Granularity | Triggers re-link | Triggers re-analyze |
|------|-------------|------------------|---------------------|
| `file` | Single file path | If parse unit | If semantic neighbor |
| `module` | Module subtree | If exports change | If boundary member |
| `symbol` | Single symbol | Rarely | If signature/calls change |

**Dependency graph (build meta):**
```
ParseUnit(file) → UsirModule → SymbolNodes → DerivedMetrics
                     ↓
              StructuralEdges
```

Each stage declares `inputs: ArtifactId[]` in manifest. Changed input → invalidate downstream only.

### 9.2 Delta Types

```
GraphDelta {
  snapshot_base:  SnapshotId
  added_nodes:    Vec<GraphNode>
  removed_nodes:  Vec<NodeId>
  added_edges:    Vec<GraphEdge>
  removed_edges:  Vec<EdgeId>
  patched_nodes:  Vec<(NodeId, patch)>
}
```

**Publish merge:** `snapshot_new = materialize(snapshot_base, GraphDelta)` — produces new immutable shards; never patches old shards.

### 9.3 Symbol Identity Across Deltas

When file changes cause symbol rename:

1. Linker emits `supersedes` edge old → new
2. Optional `same_as` when SymbolKey stable
3. Cross-snapshot queries resolve via SymbolKey index

**Why:** Git-style immutability with logical continuity for metrics and requirements.

---

## 10. Caching

### 10.1 Cache Layers

| Layer | Key | Value | Eviction |
|-------|-----|-------|----------|
| **L1 — Hot node** | NodeId | GraphNode (Arc) | LRU, ~10⁵ entries |
| **L2 — Neighborhood** | (NodeId, NeighborSpec hash) | small Vec<EdgeId> | LRU + TTL |
| **L3 — Projection** | (SnapshotId, GraphLayer) | GraphView handle | Snapshot-scoped |
| **L4 — Query result** | (query hash, snapshot) | QueryResult artifact | Content-addressed, permanent |
| **L5 — Derived shard** | Analytical tier shard | rebuild from semantic | Invalidate on upstream delta |

### 10.2 Cache Key Rule

Always include `snapshot_id` in cache keys. Never cache across snapshots without explicit version parameter.

### 10.3 Recommended Crates

| Crate | Role |
|-------|------|
| [`moka`](https://crates.io/crates/moka) | Async/sync high-perf LRU cache |
| [`lru`](https://crates.io/crates/lru) | Simple in-process LRU |

**Why moka:** Concurrent IDE + batch analysis need lock-free hot path.

---

## 11. Graph Diffing

Compare two snapshots for impact analysis, CI gates, and certification.

### 11.1 Diff Granularity

| Level | Output | Use case |
|-------|--------|----------|
| **Structural diff** | Added/removed/changed modules, files | CI "what changed" |
| **Semantic diff** | Symbol add/remove/signature change | API breaking change detection |
| **Behavioral diff** | Call edge delta | Impact analysis |
| **Metric diff** | Metric value changes | Quality gate |
| **Trace diff** | Requirement trace breakage | Compliance |

### 11.2 Diff Algorithm (Architecture)

1. **Align** nodes via `SymbolKey` (and fallback: qualified name + module + kind)
2. **Classify** nodes: `added | removed | unchanged | modified`
3. **Edge diff** on aligned ID mapping: hash edge tuples `(kind, from_key, to_key, payload_hash)`
4. **Emit** `GraphDiff` artifact (content-addressed)

```
GraphDiff {
  base:       SnapshotId
  head:       SnapshotId
  nodes:      NodeDiff[]
  edges:      EdgeDiff[]
  summary:    { breaking_changes, call_graph_delta, ... }
}
```

### 11.3 Breaking Change Detection

Rules over semantic diff:

- Public `callable` signature change → breaking
- Removed exported `symbol` → breaking
- New `depends_on` on critical module → review required

**Why diff as artifact:** Certifiers replay exact diff used for approval; CI caches diff by `(base, head)` hash.

---

## 12. Version History

### 12.1 Snapshot Chain

```
S0 (import) ← S1 (parse) ← S2 (link) ← S3 (analyze) ← ...
     ↑ parent pointer in each SnapshotManifest
```

Each snapshot is **immutable**. History is a DAG if branching (e.g. what-if transforms), but mainline is linear.

### 12.2 Time Travel API

```
trait GraphHistory {
  fn snapshot(id: SnapshotId) -> GraphView
  fn lineage(id: SnapshotId) -> Vec<SnapshotId>   // root → id
  fn diff(base: SnapshotId, head: SnapshotId) -> GraphDiff
  fn symbol_at(symbol_key: SymbolKey, snapshot: SnapshotId) -> Option<NodeId>
  fn symbol_evolution(symbol_key: SymbolKey) -> Vec<(SnapshotId, NodeId)>
}
```

**Why:** "What did we know at release 2.1?" is a certification requirement, not a nice-to-have.

### 12.3 Retention Policy

| Data | Retention |
|------|-----------|
| All snapshots | Permanent (CAS) — disk is cheap vs lost audit |
| Syntax layer shards | Optional GC after semantic materialized |
| L1/L2 caches | Ephemeral |
| Query results | Permanent if referenced by certificate |

---

## 13. Scale: Millions of Nodes

Target: **10⁶–10⁷ nodes**, **10⁷–10⁸ edges** (large monorepo + history).

### 13.1 Memory Budget Reality

| Approach | 10M nodes RAM | Verdict |
|----------|---------------|---------|
| Full adjacency in HashMap | 50–200+ GB | ❌ Reject |
| mmap CSR + lazy load | 100 MB – 2 GB working set | ✅ Primary |
| External graph DB | Variable + network | ⚠️ Phase 2 optional |

### 13.2 Strategies

| Strategy | Mechanism |
|----------|-----------|
| **Sharding** | Hash `symbol_key` or module path → 256+ shards; load one shard at a time |
| **Tier separation** | Don't load analytical tier until requested |
| **Dense local IDs** | u64 per shard, not global UUID strings in hot path |
| **CSR adjacency** | 8–16 bytes per edge compressed vs pointer-heavy objects |
| **Columnar payloads** | Scan kind/confidence without deserializing payload |
| **Roaring bitmaps** | Kind/set membership in O(1) space |
| **Traversal budgets** | `max_nodes`, `max_depth` mandatory in public API |
| **Hierarchical summary nodes** | Pre-aggregate module-level call graph for LLM context |
| **Parallel shard build** | Rayon across files/modules during materialization |
| **Incremental** | Rebuild 1–5% shards on typical commit, not 100% |

### 13.3 Order-of-Magnitude Estimates

| Component | 10M nodes, 50M edges |
|-----------|---------------------|
| CSR col + kind | ~600 MB uncompressed |
| zstd compressed shards | ~150–250 MB on disk |
| SymbolKey index | ~200 MB |
| Hot L1 cache (100k nodes) | ~50 MB |
| **Working set (typical query)** | **< 500 MB** |

### 13.4 Query Patterns at Scale

| Pattern | Strategy |
|---------|----------|
| IDE go-to-definition | SymbolKey index → single node |
| Find callers | Reverse CSR on `calls` or inverted index |
| Module dependencies | Structural layer shard only |
| Whole-program analysis | Batch, shard-parallel, spill to disk |
| LLM explain symbol | ContextSlicer BFS budget 200 nodes |

### 13.5 Recommended Rust Crates (Analytics)

| Crate | Role |
|-------|------|
| [`petgraph`](https://crates.io/crates/petgraph) | In-memory algorithms on **subgraphs** (not full graph) |
| [`rayon`](https://crates.io/crates/rayon) | Parallel shard materialization |
| [`dashmap`](https://crates.io/crates/dashmap) | Concurrent build-time dedup tables |
| [`fst`](https://crates.io/crates/fst) | Immutable symbol path index (sorted strings) |
| [`arrow`](https://crates.io/crates/arrow) / [`polars`](https://crates.io/crates/polars) | Optional: analytical aggregations over metric columns |

**Rule:** `petgraph` for **bounded** subgraph analysis; never construct full 10M node petgraph in RAM.

---

## 14. Consistency & Concurrency

| Concern | Model |
|---------|-------|
| **Snapshot publish** | Single writer; atomic manifest swap |
| **Readers** | Lock-free read of immutable shards |
| **Proposed facts** | Separate overlay or lifecycle filter at read time |
| **Concurrent analysis** | Read snapshot ID; write new derived shards; publish new snapshot |

**Why MVCC via immutability:** Eliminates graph locks; matches CAS architecture.

---

## 15. Failure Modes

| Failure | Mitigation |
|---------|------------|
| Shard corruption | Blake3 hash verify on read; manifest checksum |
| Unbounded traversal | Mandatory budgets; cancel token |
| Index drift | Index rebuild from node shards |
| SymbolKey collision | Disambiguator + human review queue |
| Delta merge conflict | Rebase delta onto latest snapshot parent |

---

## 16. Mapping to Crates

| Concern | Crate |
|---------|-------|
| Node/edge/kind types, traversal traits | `s4-graph` |
| Snapshot manifest, shard artifacts | `s4-storage` |
| Materialization from USIR | `s4-parser` + future `s4-graph-engine` |
| Metrics attachment | `s4-metrics` |
| Architecture projections | `s4-analysis` |
| Diff + history API | `s4-graph` (traits) + engine |
| Declarative query | future `s4-query` |
| Event on snapshot publish | `s4-events` |

**Future crate:** `s4-graph-engine` — implements storage, traversal, diff, cache; keeps `s4-graph` trait-only.

---

## 17. Phased Delivery

| Phase | Deliverable |
|-------|-------------|
| **G0** | Finalize NodeKind/EdgeKind enums; JSON Schema for shards |
| **G1** | mmap CSR edge store + node shard; single-snapshot read |
| **G2** | SymbolKey index; traversal API with budgets |
| **G3** | Incremental delta merge; snapshot chain |
| **G4** | GraphDiff artifact; breaking change rules |
| **G5** | S4QL subset; ContextSlicer for LLM |
| **G6** | Hierarchical summary nodes; distributed shard build |

---

## 18. Open Decisions

1. **Syntax layer** — store AST nodes in UCG or separate artifact type?
2. **Reverse CSR** — dual storage vs on-demand invert
3. **Global vs sharded NodeId** — u64 local + shard key vs single u128
4. **Hyperedges** — needed for import/export groups in v2?
5. **External graph DB** — read replica for enterprise query (Neo4j/indradb)?

---

## 19. Summary

The Universal Code Graph is:

1. A **typed multigraph** aligned with the canonical data model
2. **Layered** — physical, semantic, structural, analytical projections
3. **Immutable snapshot + delta** — Git-like history with auditable diffs
4. **Sharded CSR storage** over content-addressed artifacts — scales to 10M+ nodes
5. **Budgeted traversal API** — safe for IDE, analysis, and LLM context slicing
6. **Symbol-centered identity** — language-independent, cross-snapshot capable

Language-specific concepts are projections. Scale is handled by **never loading the whole graph** — only the shards, layers, and neighborhoods the query requires.
