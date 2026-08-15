# Language-Agnostic Parser Framework
## Tree-sitter Architecture Specification v0.1

> **Status:** Target spec.
> **Shipped:** in-process Java/Rust Tree-sitter in `s4-parser` (`extract_for_language`). WASM grammars, a separate `s4-parser-engine`, and third-party grammar plugins are **not** shipped.
> **Product:** [Porting Workflow](../guides/PORTING_WORKFLOW.md)
> **Contract crate:** `s4-parser`
> **Depends on:** [Plugin System](../plugins/PLUGIN_SYSTEM.md), [Canonical Model](../model/CANONICAL_MODEL.md), [UCG](../graph/UNIVERSAL_CODE_GRAPH.md)

---

## 1. Purpose

This document defines the **language-agnostic parser framework** for S4MP. It specifies how source text becomes durable, queryable knowledge through three layers:

```
Source bytes
    → CST (Tree-sitter concrete syntax tree)
    → UAST (Universal AST — language-normalized syntax)
    → USIR (Universal Semantic IR — language-agnostic semantics)
    → UCG (graph materialization — outside this document)
```

The **core never imports language grammars or lowering logic**. Languages are added via **parser plugins** that register Tree-sitter grammars and USIR lowering adapters.

---

## 2. Why Tree-sitter

| Requirement | Tree-sitter | Alternative (e.g. LSP, custom, LLVM frontends) |
|-------------|-------------|-----------------------------------------------|
| **Incremental parse** | Native `Tree.edit()` + re-parse changed regions | Usually full re-parse |
| **Error tolerance** | Produces usable tree on broken code | Often fail-stop |
| **Multi-language** | 40+ maintained grammars, uniform C API | Per-language toolchain |
| **Performance** | Sub-millisecond reparse on edits | Varies widely |
| **Embedding** | Single C library; Rust bindings mature | Heavy per-language deps |
| **Stable node model** | Named nodes + fields + byte ranges | Heterogeneous ASTs |
| **Sandbox-friendly** | Grammar is data + native lib; WASM path exists | Full compiler frontends too heavy |
| **No core coupling** | Grammar loaded at runtime per plugin | rustc/clang tied to language |

**Why not use Tree-sitter as the semantic IR:** Tree-sitter nodes are **syntax** (`function_item`, `call_expression`). S4MP semantics (`Callable`, `Contract`, `Dependency`) require a **lowering pass** — implemented in language plugins, not core.

**Why not LSP alone:** LSP is IDE-oriented, process-per-language, inconsistent across servers, and poor for batch/batch-certification pipelines. Tree-sitter provides a **local, deterministic, embeddable** syntax foundation; LSP may supplement (future) for type-checker-grade semantics.

**Recommended Rust crates:**

| Crate | Role |
|-------|------|
| [`tree-sitter`](https://crates.io/crates/tree-sitter) | Parser runtime |
| [`tree-sitter-language`](https://crates.io/crates/tree-sitter-language) | Grammar handle type |
| Language grammars (`tree-sitter-rust`, etc.) | **Plugin deps only** — never in `s4-core` |
| [`streaming-iterator`](https://crates.io/crates/streaming-iterator) | Zero-copy tree walks (transitive) |

---

## 3. Architectural Boundaries

```
┌─────────────────────────────────────────────────────────────┐
│  s4-parser (traits): ParsePipeline, UAST, USIR contracts  │
├─────────────────────────────────────────────────────────────┤
│  s4-parser-engine: TreeHost, cache, incremental scheduler   │
├─────────────────────────────────────────────────────────────┤
│  Parser plugins (per language):                           │
│    GrammarRegistration + LoweringAdapter → USIR           │
╞═════════════════════════════════════════════════════════════╡
│  tree-sitter C library + language .so grammar               │
└─────────────────────────────────────────────────────────────┘
```

**Core (`s4-parser` traits only) knows:**
- `ParseUnit`, `ParseResult`, `UastNode`, `UsirModule`
- `Parser` / `SyntaxParser` / `LoweringAdapter` trait signatures
- Error and cache key types

**Core never knows:**
- `tree_sitter::Language` concrete handles (engine internal)
- Language-specific node type strings (`function_item`, etc.) in stable API
- Regex, nom, or custom parsers in core path

---

## 4. Parser Traits

Three trait layers mirror the three output layers.

### 4.1 Base — `SyntaxParser` (Plugin role: Parser)

```rust
/// Language plugin entry: owns grammar + lowering registration.
trait SyntaxParser: Plugin {
    /// Grammar identity and version this parser provides.
    fn grammar(&self) -> GrammarInfo;

    /// Parse source into CST + UAST artifacts (via host context).
    fn parse_syntax(&self, ctx: &mut InvokeContext, unit: &ParseUnit) -> Result<ParseSyntaxOutput>;

    /// Lower UAST (or CST ref) to USIR module artifact.
    fn lower_to_usir(&self, ctx: &mut InvokeContext, input: &LowerInput) -> Result<()>;
}
```

**Why combined in one plugin:** Grammar and lowering stay version-synced (`tree-sitter-rust 0.21` matches Rust lowering rules for that grammar).

### 4.2 Engine — `TreeHost` (internal, `s4-parser-engine`)

```rust
/// Tree-sitter runtime owned by platform — not implemented by plugins.
trait TreeHost: Send + Sync {
    fn parse(
        &self,
        grammar: GrammarHandle,
        source: SourceBuffer,
        previous: Option<ParsedTreeState>,
        edits: &[SourceEdit],
    ) -> Result<ParsedTreeState>;

    fn language_for(&self, language_id: &LanguageId) -> Result<GrammarHandle>;
}
```

### 4.3 Lowering — `LoweringAdapter`

```rust
/// Maps CST/UAST nodes to USIR. Implemented by language plugin.
trait LoweringAdapter: Send + Sync {
    fn language_id(&self) -> &LanguageId;

    /// Map Tree-sitter node type string → UastKind (may be many-to-one).
    fn map_node_kind(&self, ts_kind: &str) -> UastKind;

    /// Emit USIR entities/relations from UAST subtree root.
    fn lower_module(
        &self,
        module: &UastModule,
        ctx: &mut LoweringContext,
    ) -> Result<UsirModule>;
}
```

### 4.4 Optional — `IncrementalParseSession`

```rust
/// Per-file mutable session for repeated edits (IDE loop).
trait IncrementalParseSession: Send {
    fn apply_edit(&mut self, edit: SourceEdit) -> Result<()>;
    fn reparse(&mut self) -> Result<ParsedTreeState>;
    fn snapshot(&self) -> ParsedTreeSnapshot;  // cheap clone for readers
}
```

**Why session trait:** IDE and watch mode hold parser state across keystrokes; batch CI creates fresh sessions per file.

---

## 5. Universal AST (UAST)

The UAST sits between Tree-sitter's **language-specific CST** and **USIR semantics**. It is the **canonical syntax representation** stored in the artifact store.

### 5.1 Design Goals

1. **Language-neutral node kinds** — `FunctionDecl`, not `function_item`
2. **Lossless source anchoring** — every node has byte range + optional points
3. **Preserve error nodes** — `Error`, `Missing` as first-class kinds
4. ** Serializable & diffable** — artifact in CAS
5. **Cheap to traverse** — arena layout, parent/child indices

### 5.2 UAST Node Model

```
UastModule {
  language_id:   LanguageId
  file_ref:      ArtifactId          // source bytes
  root:          UastNodeId
  nodes:         UastNode[]          // arena flat list
  errors:        ParseDiagnostic[]
  grammar:       { name, version }
}

UastNode {
  id:            UastNodeId          // index into arena
  kind:          UastKind
  child_range:   Range<UastNodeId>   // contiguous child slice
  field_slots:   Vec<(UastField, UastNodeId)>  // named fields
  span:          SourceSpan            // byte + row/col
  ts_kind:       Option<String>        // original TS type (debug/extensions)
  is_error:      bool
}

UastKind (frozen enum + extension):
  Module | Import | FunctionDecl | ClassDecl | InterfaceDecl
  | TraitDecl | TypeDecl | VariableDecl | Parameter | Block
  | CallExpr | ReferenceExpr | Literal | Attribute | Comment
  | Error | Missing | Unknown
  | Extension(ExtensionKindId)
```

### 5.3 CST → UAST Normalization

Language plugin supplies **`NodeKindMap`**:

```toml
# embedded in parser plugin manifest
[[node_map]]
ts_kind = "function_item"
uast_kind = "FunctionDecl"
fields = { name = "name", body = "body", parameters = "parameters" }
```

**Why not skip UAST and lower CST directly:** Normalization decouples USIR lowering from grammar renames (`function_item` → `function_definition` in TS grammar bump). USIR lowering reads stable `UastKind`.

### 5.4 UAST vs USIR vs UCG Syntax Layer

| Layer | Content | Consumer |
|-------|---------|----------|
| **CST** | Ephemeral Tree-sitter tree | Parser engine only |
| **UAST** | Normalized syntax artifact | IDE, formatters, syntax queries |
| **USIR** | Symbols, callables, types | Linker, analyzers, UCG semantic tier |
| **UCG Syntax** | Optional projection of UAST | Graph syntax layer (UCG doc) |

---

## 6. Error Handling

### 6.1 Error Categories

| Category | Source | Handling |
|----------|--------|----------|
| **Lexical/syntax (TS)** | Tree-sitter `ERROR`, `MISSING` nodes | Capture in UAST; parse continues |
| **Grammar load** | Invalid/missing `.so` | Fail plugin load; don't crash host |
| **Lowering** | Unmapped node, ambiguous symbol | `ParseDiagnostic` + partial USIR |
| **Resource** | Timeout, memory cap | Cancel; return last good incremental tree |
| **I/O** | Missing file artifact | Structured `ParseError` to pipeline |

### 6.2 Diagnostic Model

```
ParseDiagnostic {
  level:     error | warning | info
  code:      string              // "syntax/missing-token", "lower/unresolved-type"
  message:   string
  span:      SourceSpan
  node:      Option<UastNodeId>
  recovery:  recovered | partial | failed
}
```

**Rules:**
1. **Never panic** on bad source — ship best-effort UAST/USIR
2. **Errors are artifacts** — stored alongside tree for certification audit
3. **USIR from error-tolerant parse** marks affected entities `confidence < 1.0`
4. **Fatal only for infrastructure** — grammar missing, sandbox violation

### 6.3 Error Node Policy

Tree-sitter `ERROR` nodes become `UastKind::Error` with children preserved. Lowering **skips or marks** error subtrees; sibling branches still produce USIR.

---

## 7. Caching

### 7.1 Cache Key Hierarchy

| Key | Components | Invalidation |
|-----|------------|--------------|
| **Source** | `blake3(content)` | File edit |
| **Grammar** | `(language_id, grammar_version)` | Plugin upgrade |
| **Syntax cache** | `(source_hash, grammar_version)` | Either component |
| **UAST artifact** | Content-addressed | Immutable once written |
| **USIR artifact** | `(uast_hash, lowering_version)` | Lowering adapter bump |
| **Session cache** | `(file_path, snapshot_id)` | In-memory IDE only |

### 7.2 Cache Layers

```
L1  IncrementalParseSession     (in-memory, per open file, IDE)
L2  ParsedTreeState              (CST handles + tree id, short-lived)
L3  UAST artifact               (CAS, permanent per source+grammar)
L4  USIR artifact               (CAS, permanent per uast+lowerer)
L5  Parse result index          (file_path → latest artifact ids)
```

**Recommended crates:**
- [`moka`](https://crates.io/crates/moka) — concurrent L2/L5
- [`blake3`](https://crates.io/crates/blake3) — source hashing (in `s4-core`)

### 7.3 Cache-Through Flow

```
parse(unit):
  if cas.contains(source_hash, grammar) → return cached UAST id
  else run tree-sitter → normalize → write UAST artifact → return id

lower(uast_id):
  if cas.contains(uast_hash, lowerer_version) → return cached USIR id
  else run LoweringAdapter → write USIR artifact
```

**Why content-addressed syntax cache:** Identical file content across branches shares UAST — common in monorepos.

---

## 8. Incremental Parsing

### 8.1 Tree-sitter Incremental Model

Tree-sitter supports:

1. Hold previous `Tree` + `source` bytes
2. Apply `InputEdit` (byte range replace)
3. `parse(source, Some(old_tree), edits)` → new tree reuses unchanged subtrees

```
SourceEdit {
  start_byte, old_end_byte, new_end_byte,
  start_point, old_end_point, new_end_point
}
```

### 8.2 S4MP Incremental Architecture

```
FileWatch / IDE
    │
    ▼
IncrementalParseSession (per file)
    │ apply_edit(edit)
    ▼
TreeHost::parse(grammar, source, Some(prev), edits)
    │
    ├──► CST (updated Tree-sitter tree)
    │
    ├──► Diff: changed node ranges (TS changed_ranges API)
    │
    ▼
Partial UAST rebuild (only affected subtrees)  [optimization phase]
    │
    ▼
Selective USIR invalidation → UCG delta (UCG doc)
```

### 8.3 Invalidation Granularity

| Change type | Reparse scope | USIR impact |
|-------------|---------------|-------------|
| Comment/whitespace | TS incremental (minimal) | None if lowerer ignores |
| Function body edit | TS incremental subtree | That callable + call edges |
| Import added | TS incremental + imports list | Module dependency edges |
| Grammar version bump | Full reparse | Full file USIR |

### 8.4 Batch vs Interactive Mode

| Mode | Session | Incremental |
|------|---------|-------------|
| **CI / batch import** | Fresh per file; syntax cache only | No cross-invocation tree |
| **IDE / watch** | Long-lived `IncrementalParseSession` | Full TS incremental |
| **Commit delta** | Git diff → synthetic `SourceEdit[]` per file | Incremental from last snapshot tree state loaded from cache if available |

### 8.5 Incremental State Persistence (Optional)

Store serialized `ParsedTreeState` in CAS for large files to avoid cold-start full parse:

```
ParsedTreeStateArtifact {
  source_hash, grammar_version,
  tree_blob: ArtifactId,    // tree-sitter serialized tree
}
```

**Recommended:** Tree-sitter `Tree` can be copied; serialized form enables snapshot restore (evaluate `tree-sitter` print/dot export vs custom postcard encoding of edit history).

---

## 9. Performance Strategy

### 9.1 Targets

| Scenario | Target |
|----------|--------|
| Cold parse 1K LOC file | < 10 ms |
| Incremental keystroke reparse | < 5 ms |
| Batch 10K files (CI) | Parallel, limited by cores |
| UAST normalization | < 2× TS parse time |
| USIR lowering | Language-dependent; budget 50 ms/file typical |

### 9.2 Strategies

| Strategy | Mechanism |
|----------|-----------|
| **Parallel file parsing** | `rayon` thread pool; one session per file |
| **Grammar once per thread** | Thread-local `tree_sitter::Parser` with `set_language` |
| **Zero-copy source** | `Arc<str>` or `mmap` source buffer; spans are offsets |
| **Skip UAST when cached** | CAS hit before TS invoke |
| **Lazy lowering** | Syntax-only stages for metrics that don't need USIR |
| **Changed-ranges only** | TS `Tree::changed_ranges` → partial lower (phase 2) |
| **Parse budget** | Cancel long parses; emit partial result |
| **Work stealing queue** | Host schedules ParseUnits across cores |

**Recommended crates:**
- [`rayon`](https://crates.io/crates/rayon) — parallel batch
- [`memmap2`](https://crates.io/crates/memmap2) — large file sources

### 9.3 Anti-Patterns

| Anti-pattern | Why avoid |
|--------------|-----------|
| Global mutex around single `Parser` | Destroys parallelism |
| String clone per node | Use arena + spans |
| Full USIR rebuild on whitespace edit | Use incremental + comment-skipping lowerer |
| Loading all grammars at startup | Lazy load on first file extension match |

---

## 10. Memory Layout

### 10.1 Source Buffer

```
SourceBuffer {
  bytes:     Arc<[u8]>     // or mmap handle
  encoding:  Utf8 | Utf16Le | ...
  path:      Option<String>  // metadata only
}
```

**Span model:**
```
SourceSpan {
  start_byte: u32,
  end_byte:   u32,
  start:      { row: u32, column: u32 },
  end:        { row, column },
}
```

All offsets **byte-based** (Tree-sitter native); line/col cached for diagnostics.

### 10.2 UAST Arena Layout

Flat arena (cache-friendly):

```
nodes: Vec<UastNode>           // index = UastNodeId
children: Vec<UastNodeId>      // child_range slices into this
fields: Vec<(u16, UastNodeId)> // parallel to nodes or inline small vec
```

**Why flat arena:** DFS traversal is sequential memory access; serialization is one blob; diffing is range-friendly.

Estimated size: ~40–64 bytes per node → 1M nodes ≈ 40–64 MB (upper bound for huge generated files; shard by file in practice).

### 10.3 CST Lifetime

| Object | Lifetime | Storage |
|--------|----------|---------|
| `tree_sitter::Tree` | Session or until next parse | Heap (TS allocator) |
| `tree_sitter::Node` | Borrowed from Tree | Zero-copy view |
| UAST | `'static` in artifact | CAS |

**Rule:** Never persist raw `Tree` as primary truth — always materialize UAST artifact for reproducibility.

### 10.4 USIR Layout

Existing `UsirModule` vec-based model; future columnar shard if needed. Produced once per UAST; stored as CAS artifact.

---

## 11. Thread Safety

### 11.1 Tree-sitter Concurrency Model

| Type | Send | Sync | Notes |
|------|------|------|-------|
| `tree_sitter::Parser` | Yes | **No** | One parser per thread |
| `tree_sitter::Tree` | Yes | Yes | Immutable after parse |
| `tree_sitter::Node` | — | — | Copy handle; tied to Tree lifetime |
| `Language` | Yes | Yes | Immutable grammar |

### 11.2 S4MP Threading Architecture

```
TreeHost (Sync)
  └── ParserPool
        ├── thread_local: Parser + loaded Language
        └── dispatch parse jobs via rayon

IncrementalParseSession: NOT Sync
  └── pinned to IDE thread OR mutex per file path

ParsedTreeSnapshot: Arc<Tree> + Arc<SourceBuffer>  → Sync, shareable read-only

UAST / USIR artifacts: immutable CAS → freely concurrent reads
```

### 11.3 Concurrency Rules

1. **One mutable `Parser` per thread** — never share across threads
2. **Share immutable trees** — `Arc<ParsedTreeSnapshot>` for analysis workers
3. **Session map** — `DashMap<Path, Mutex<IncrementalParseSession>>` for IDE
4. **Lowering is pure** — `LoweringAdapter::lower_module` takes `&UastModule`, parallelizable across files
5. **Plugin invoke** — host serializes or pools; WASM plugins single-threaded per instance

**Recommended crates:**
- [`dashmap`](https://crates.io/crates/dashmap) — concurrent session map
- [`parking_lot`](https://crates.io/crates/parking_lot) — lighter mutex for sessions

---

## 12. Adding a New Language (Zero Core Changes)

```
1. Create plugin crate: plugins/s4-parser-python
2. Depend on tree-sitter-python (plugin scope only)
3. Implement SyntaxParser + LoweringAdapter
4. Ship s4-plugin.toml:
     roles = ["parser"]
     languages = ["python"]
     file_patterns = ["**/*.py"]
5. Provide NodeKindMap for CST → UAST
6. Register via inventory (static) or WASM artifact (dynamic)
7. Add to workspace s4.toml — no core rebuild required (dynamic path)
```

**Core changes required:** **None** — registry resolves new plugin by `LanguageId` + glob.

---

## 13. Artifact Outputs (Parse Pipeline)

| Stage | Artifact kind | Content |
|-------|---------------|---------|
| Parse syntax | `syntax_tree` | UAST module + diagnostics |
| Lower | `usir_module` | USIR entities/relations |
| Optional | `cst_debug` | DOT/JSON dump (dev only) |

All emitted via `InvokeContext.outputs` with provenance `source_type: parse`.

---

## 14. Integration Points

| System | Integration |
|--------|-------------|
| **Plugin host** | `SyntaxParser` as `Parser` role |
| **CAS (`s4-storage`)** | Source, UAST, USIR artifacts |
| **UCG** | USIR → graph materializer |
| **Incremental (UCG doc)** | USIR delta → graph delta |
| **Metrics plugins** | May consume UAST only (complexity without full USIR) |
| **Events (`s4-events`)** | `ParseCompleted { file, uast, usir }` |

---

## 15. Phased Delivery

| Phase | Deliverable |
|-------|-------------|
| **T0** | Finalize UAST schema + trait signatures in `s4-parser` |
| **T1** | TreeHost batch parse; full UAST materialization |
| **T2** | Rust plugin reference: grammar + lowering |
| **T3** | Syntax cache (CAS); parallel batch |
| **T4** | IncrementalParseSession (IDE) |
| **T5** | Changed-range partial lowering |
| **T6** | WASM grammar plugins |

---

## 16. Open Decisions

1. **Persist serialized Tree** for cold incremental vs always full reparse from CAS UAST
2. **UAST columnar** vs arena vec at scale
3. **Tree-sitter vs tree-sitter WASM** for grammar distribution in sandbox
4. **Partial lowering API** — node-level vs module-level invalidation
5. **Non-Tree-sitter fallback** — allow custom `SyntaxParser` without TS for DSLs?

---

## 17. Summary

| Topic | Decision |
|-------|----------|
| **Engine** | Tree-sitter — incremental, error-tolerant, multi-language, embeddable |
| **Layers** | CST (ephemeral) → UAST (artifact) → USIR (semantic artifact) |
| **Traits** | `SyntaxParser`, `TreeHost`, `LoweringAdapter`, `IncrementalParseSession` |
| **New languages** | Parser plugin only — grammar + NodeKindMap + lowerer |
| **Errors** | Best-effort trees; diagnostics as artifacts; partial USIR |
| **Cache** | Content-addressed UAST/USIR; session cache for IDE |
| **Incremental** | Tree-sitter edits + selective invalidation downstream |
| **Performance** | Parallel per-file; thread-local parsers; zero-copy spans |
| **Memory** | Flat UAST arena; Arc source; immutable CAS artifacts |
| **Threads** | Parser !Sync; Tree/UAST/USIR shareable immutably |

The core orchestrates and caches. Plugins supply grammars and lowering. Tree-sitter supplies fast, incremental syntax. Semantics remain in USIR — language-independent and certification-ready.
