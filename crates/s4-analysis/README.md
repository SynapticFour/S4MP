# s4-analysis

Architecture extraction, feature extraction, and cross-graph analysis.

## Public API

| Module | Purpose |
|--------|---------|
| `finding` | `Finding`, `Severity` |
| `lowering` | `usir_to_graph` — USIR modules → semantic graph |
| `correspondence` | `GraphId`, `CorrespondenceEntry`, `suggest_correspondences`, `load/save/merge` |
| `diff_report` | `DiffReport`, `build_diff_report`, `render_markdown` |
| `pass` | `Pass`, `PassPipeline`, `PORTING_PASS_ORDER` (`s4 analyze` runs this pipeline) |
| `architecture` | `ArchitectureAnalyzer`, `Boundary`, `Pattern` (contracts) |
| `feature` | `Feature`, `FeatureExtractor` trait (contracts) |
| `pipeline` | `AnalysisPipeline` trait (contract) |

## Porting pipeline role

| Function | CLI step |
|----------|----------|
| `usir_to_graph` | `s4 graph` |
| `suggest_correspondences` | `s4 map suggest` |
| `merge_correspondences` | Preserves manual rows on re-suggest |
| `build_diff_report` + `render_markdown` | `s4 diff` |

See [Porting Workflow Guide](../../docs/guides/PORTING_WORKFLOW.md) for end-to-end usage.

### Correspondence heuristics (v2)

- Tokenized name Jaccard, optional signature Jaccard (`0.6 * name + 0.4 * signature`)
- Tokens are computed **once per node**; an inverted token index prunes candidates
- Assignment is **exclusive** (greedy by descending score)
- Empty/`<anonymous>` labels are skipped; empty-empty Jaccard is 0
- Threshold ≥ 0.5 → `Diverged` (never auto-`Ported`)
- Unmatched Java nodes → `MissingInTarget`; unmatched Rust → `ExtraInTarget`
- `coverage_pct` is **ported callables / Java callables** (types do not inflate coverage)

## Tier

2 — Capabilities
