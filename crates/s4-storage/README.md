# s4-storage

Content-addressed artifact storage (CAS) for immutable S4MP artifacts.

## Responsibility

Defines how immutable artifacts are read, written, and referenced via manifests. All cross-crate data exchange uses content-addressed [`ArtifactId`](../../crates/s4-core/src/id.rs) values.

## Public API

| Module | Purpose |
|--------|---------|
| `artifact` | `Artifact`, `ArtifactKind` |
| `manifest` | `Manifest`, `ManifestRef` |
| `store` | `StoreReader`, `StoreWriter`, `Store` traits |
| `filesystem` | `FileSystemStore` — JSON file-backed CAS |

## FileSystemStore

The default v1 store implementation used by `s4-cli`:

- Root: `.s4/store/` (one JSON file per artifact)
- Path pattern: `.s4/store/<blake3-hex>.json`
- Content-addressed: `Artifact::id()` / `canonical_bytes()` — compact JSON of the envelope (the bytes written to disk)
- Atomic writes: temp file + rename
- Idempotent writes: existing IDs are not overwritten
- `StoreWriter::write_at` writes a **CAS pointer** at the index id and stores the envelope under its content hash. `read` follows the pointer.

```rust
use s4_storage::{FileSystemStore, StoreWriter};

let mut store = FileSystemStore::workspace(".")?;
let id = store.write(&artifact)?;
```

Used by the [Porting Workflow](../../docs/guides/PORTING_WORKFLOW.md) for USIR modules, graph projections, physical snapshots, and correspondence maps.

## Artifact kinds (v1)

| Kind | Used by |
|------|---------|
| `PhysicalSnapshot` | `s4-project::snapshot_physical` |
| `UsirModule` | `s4-parser` plugins |
| `GraphProjection` | `s4-cli graph` |
| `CorrespondenceMap` | `s4-analysis::save_correspondence_map` |
| `UsirCache` | parser per-file USIR index (`write_at`) |

## Tier

1 — Infrastructure
