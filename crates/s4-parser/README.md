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
| `pipeline` | `ParsePipeline` trait |

## Tier

2 — Capabilities
