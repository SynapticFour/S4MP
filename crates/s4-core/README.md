# s4-core

Foundation crate for the SynapticFour Method Platform.

## Responsibility

Shared identifiers, error types, schema versioning, timestamps, and the workspace prelude. No domain logic.

## Public API

| Module | Purpose |
|--------|---------|
| `error` | `S4Error`, `Result` (`InvalidId`, `InvalidInput`, `Storage`, `Plugin`, `External`, `CheckFailed`, `SchemaVersionMismatch`) |
| `id` | `ArtifactId`, `EntityId` (graph alias + node), `PluginId`, `ProjectId` |
| `language` | `LanguageId` |
| `maturity` | `MATURITY`, `MATURITY_NOTICE` |
| `time` | `utc_rfc3339`, `unix_secs_to_rfc3339` |
| `version` | `SchemaVersion`, `ApiVersion` |
| `prelude` | Common re-exports |

## Dependencies

`blake3`, `serde`, `thiserror`.

## Tier

0 — Foundation
