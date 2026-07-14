# s4-project

Project workspace, source registration, and ingestion.

## Responsibility

Defines how S4MP projects are opened, configured, locked, and referenced by snapshot. Provides source tree resolution (local paths and Git clones) for the CLI porting pipeline.

## Public API

| Module | Purpose |
|--------|---------|
| `workspace` | `Workspace` trait |
| `config` | `ProjectConfig` |
| `lockfile` | `Lockfile` |
| `ingest` | `SourceIngestor`, `DefaultSourceIngestor`, `ResolvedSource`, `snapshot_physical` |
| `snapshot` | `SnapshotRef` |
| `source` | `SourceOrigin`, `SourceRef` |

## Source ingestion

`DefaultSourceIngestor` resolves [`SourceRef`](src/source.rs) definitions:

| Origin | Behavior |
|--------|----------|
| `Local { path }` | Use directory as-is |
| `Git { url, git_ref, subpath }` | Clone/fetch into `.s4/cache/<alias>/`, optional checkout + subpath |

`snapshot_physical(root)` walks a directory tree, Blake3-hashes every file, skips `.git/`, `target/`, `node_modules/`, `.s4/`.

## CLI usage

Registered via `s4 source add` → stored in `.s4/sources.json`. See [Porting Workflow Guide](../../docs/guides/PORTING_WORKFLOW.md).

```bash
cargo run -p s4-cli -- source add gatk-java-hc \
  --git https://github.com/broadinstitute/gatk.git \
  --subpath src/main/java/.../haplotypecaller \
  --lang java
```

## Tier

1 — Infrastructure
