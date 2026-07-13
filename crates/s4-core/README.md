# s4-core

Foundation crate for the SynapticFour Method Platform.

## Responsibility

Shared identifiers, error types, schema versioning, and the workspace prelude. No domain logic.

## Public API

| Module | Purpose |
|--------|---------|
| `error` | `S4Error`, `Result` |
| `id` | `ArtifactId`, `EntityId`, `PluginId`, `ProjectId` |
| `version` | `SchemaVersion`, `ApiVersion` |
| `prelude` | Common re-exports |

## Dependencies

None (internal).

## Tier

0 — Foundation
