# s4-analysis

Architecture extraction, feature extraction, and cross-graph analysis.

## Public API

| Module | Purpose |
|--------|---------|
| `finding` | `Finding`, `Severity` |
| `lowering` | `usir_to_graph` — USIR modules → semantic graph |
| `correspondence` | `GraphId`, `CorrespondenceEntry`, `suggest_correspondences`, `load/save/merge` |
| `diff_report` | `DiffReport`, `build_diff_report`, `render_markdown` |
| `architecture` | `ArchitectureAnalyzer`, `Boundary`, `Pattern` |
| `feature` | `Feature`, `FeatureExtractor` trait |
| `pipeline` | `AnalysisPipeline` trait |

## Porting pipeline role

| Function | CLI step |
|----------|----------|
| `usir_to_graph` | `s4 graph` |
| `suggest_correspondences` | `s4 map suggest` |
| `merge_correspondences` | Preserves manual rows on re-suggest |
| `build_diff_report` + `render_markdown` | `s4 diff` |

See [Porting Workflow Guide](../../docs/guides/PORTING_WORKFLOW.md) for end-to-end usage.

### Correspondence heuristics (v1)

- Tokenized name Jaccard similarity (callable↔callable, type↔type)
- Threshold ≥ 0.5 → `Diverged` + `NameHeuristic` (never auto-`Ported`)
- Unmatched Java nodes → `MissingInTarget`
- Unmatched Rust nodes → `ExtraInTarget`

## Tier

2 — Capabilities
