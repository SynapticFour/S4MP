# s4-plugin

Plugin system contracts — the stable extension surface for S4MP.

## Responsibility

Defines plugin manifests, capability declarations, and specialized plugin traits (importer, parser, analyzer, reasoner, verifier).

## Public API

| Module | Purpose |
|--------|---------|
| `plugin` | Base `Plugin` trait, `InvocationContext` |
| `manifest` | `PluginManifest`, `CapabilitySet` |
| `importer` | `Importer` trait |
| `parser` | `Parser` trait |
| `analyzer` | `Analyzer` trait |
| `reasoner` | `Reasoner` trait |
| `verifier` | `Verifier` trait |
| `host` | `PluginHost` trait |

## Tier

1 — Infrastructure
