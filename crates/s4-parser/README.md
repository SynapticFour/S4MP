# s4-parser

Universal parsing orchestration and in-tree Tree-sitter frontends.

## Responsibility

Defines parse units, USIR module contracts, and parse pipeline traits. Java and Rust v1 frontends live in this crate (`plugins`); they implement `ParsePipeline`, not `s4_plugin::Parser`.

## Public API

| Module | Purpose |
|--------|---------|
| `language` | `LanguageId` (re-export of `s4_core::LanguageId`) |
| `unit` | `ParseUnit` |
| `usir` | `UsirModule`, entity/relation kinds |
| `pipeline` | `ParsePipeline`, `ParseContext` |
| `plugins` | `JavaParser`, `RustParser`, `extract_for_language`, `parse_all_parallel` |

Used by `s4 graph`. v1 extraction is heuristic (Tree-sitter `method_invocation` / `call_expression` names, no full type system). Overloads of the same name are kept; cross-module unresolved calls link only when the callee name is unique.

`parse_all_parallel` extracts with bounded threads then persists sequentially. When `ParseUnit::source_hash` is set, USIR artifacts are reused from CAS via a `usir_cache` index record. `s4 graph build` also skips the whole parse when the physical snapshot hash is unchanged.

## Tier

2 — Capabilities
