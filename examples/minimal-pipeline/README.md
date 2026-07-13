# Minimal Pipeline Example

This example demonstrates the intended end-to-end S4MP pipeline:

1. Import a Git repository (`s4mp-importer-git`)
2. Parse Rust sources (`s4mp-parser-rust`)
3. Link USIR modules
4. Materialize the knowledge graph
5. Run analysis and query

```bash
# From workspace root (once implemented):
cargo run -p s4mp-cli -- init ./my-project
cargo run -p s4mp-cli -- analyze
cargo run -p s4mp-cli -- query --expr all
```

See [docs/architecture/ARCHITECTURE.md](../../docs/architecture/ARCHITECTURE.md) for the full platform design.
