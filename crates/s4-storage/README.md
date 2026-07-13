# s4-storage

Content-addressed artifact storage (CAS) contracts.

## Responsibility

Defines how immutable artifacts are read, written, and referenced via manifests. Implementations live outside this crate.

## Public API

| Module | Purpose |
|--------|---------|
| `artifact` | `Artifact`, `ArtifactKind` |
| `manifest` | `Manifest`, `ManifestRef` |
| `store` | `StoreReader`, `StoreWriter`, `Store` |

## Tier

1 — Infrastructure
