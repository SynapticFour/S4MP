# s4-plugin

Plugin system contracts — the stable extension surface for S4MP.

## Responsibility

Plugin **manifests and an in-process registry** — not a loadable plugin runtime.

Java/Rust frontends are first-party code in `s4-parser` (`extract_for_language`). `s4 plugin list` prints the same builtins. WASM / third-party load is deferred (ADR-016).

## Public API

| Module | Purpose |
|--------|---------|
| `plugin` | Base `Plugin` trait, `InvocationContext` |
| `manifest` | `PluginManifest`, `CapabilitySet`, `PluginCapability` |
| `importer` | `Importer` trait |
| `parser` | `Parser` trait |
| `analyzer` | `Analyzer` trait |
| `reasoner` | `Reasoner` trait |
| `verifier` | `Verifier` trait |
| `host` | `PluginHost` trait |
| `in_process` | `InProcessPluginHost` (Phase 6 builtins) |

## Tier

1 — Infrastructure
