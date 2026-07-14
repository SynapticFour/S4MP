# s4-parser

Universal parsing orchestration contracts.

## Responsibility

Defines parse units, language identifiers, USIR module contracts, and parse pipeline traits. Language-specific parsing is delegated to plugins.

## Public API

| Module | Purpose |
|--------|---------|
| `language` | `LanguageId` |
| `unit` | `ParseUnit` |
| `usir` | `UsirModule`, entity/relation kinds |
| `pipeline` | `ParsePipeline`, `ParseContext` |
| `plugins` | `JavaParser`, `RustParser` (tree-sitter v1 frontends) |

Used by `s4 graph` in the [Porting Workflow](../../docs/guides/PORTING_WORKFLOW.md). v1 extraction is heuristic (intra-file call detection, no full type system).

## Tier

2 — Capabilities
