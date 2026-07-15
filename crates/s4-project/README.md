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
| `Git { url, git_ref, subpath }` | Clone/fetch into `.s4/cache/<alias>/`, optional checkout + subpath. When `subpath` is set, uses sparse checkout with `--filter=blob:none` on first clone (Git >= 2.27). |

`snapshot_physical(root)` walks a directory tree, Blake3-hashes every file, skips `.git/`, `target/`, `node_modules/`, and workspace metadata under `.s4/` (not `.s4/cache/` git source trees).

## CLI usage

Registered via `s4 source add` → stored in `.s4/sources.json`. See [Porting Workflow Guide](../../docs/guides/PORTING_WORKFLOW.md).

```bash
cargo run -p s4-cli -- source add gatk-java-hc \
  --git https://github.com/broadinstitute/gatk.git \
  --subpath src/main/java/.../haplotypecaller \
  --lang java
```

### Manual test (Git + subpath)

Sparse-checkout clone requires network access and Git >= 2.27; it is not exercised in CI.
After changing ingestion logic, verify locally:

```bash
cargo run -p s4-cli -- source add test-sparse \
  --git https://github.com/broadinstitute/gatk.git \
  --subpath src/main/java/org/broadinstitute/hellbender/tools/walkers/haplotypecaller \
  --lang java
ls .s4/cache/test-sparse/src/main/java/org/broadinstitute/hellbender/tools/walkers/haplotypecaller
```

## Tier

1 — Infrastructure
