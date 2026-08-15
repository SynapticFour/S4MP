# Changelog

All notable changes to S4MP are documented here. The crate version remains `0.1.0` (unreleased).

## Unreleased

### Breaking

- Default `s4 certify` policy requires at least one manually confirmed `Ported` row (`min_ported:1`). Heuristic-only maps write an `Invalid` certificate and exit non-zero.
- `InProcessPluginHost::with_builtins` returns `Result` and fails if a builtin manifest is incompatible.
- Git remotes are allowlisted (`https`/`http`/`ssh`/`git@host:path`). Refs and URLs starting with `-` are rejected. `git clone` uses `--` before the URL.
- Physical snapshots fail when they exceed 50k files, 32 MiB per file, or 512 MiB total.
- `git rev-parse HEAD` failure after clone is an error (no silent `None`).
- `s4 map confirm` / `reject` take `--id` (unique prefix) or `--name`, optionally scoped with `--java`/`--rust`. Confirming extra or missing rows is an error.
- Diff reports are English and print `id=` plus Java↔Rust pairings; German section titles are gone.

### Changed

- README and GitHub description describe the shipped CLI, not a certification platform.
- Call edges come from Tree-sitter `method_invocation` / `call_expression` nodes, not a `name(` text scan.
- `write_at` stores CAS pointers; envelopes live only under their Blake3 id.
- `s4 reason` includes the operator prompt in the heuristic claim text.
- Certificate ids are derived from the verification-run bytes, not the constant `1`.
- CI uses `--locked`, does not cancel in-progress jobs on `main`, and runs MSRV on pull requests.
- Local `ci-check.sh` matches CI (fmt, clippy, test, doc, tiers, deny if installed).
- Correspondence `display_name` is `Java ↔ Rust` using qualified names (`Type.method` / `Type::method`) when known.
- CLI `--help` leads with the port-map loop; `require` / `knowledge` / `plugin` / `reason` are hidden satellite commands.

### Added

- ADR-001 through ADR-012 as records (previously index-only).
- Dual-license root `LICENSE` file.
- `validate_git_url` / `validate_git_ref`.
- `extract_for_language` as the single Java/Rust dispatch table.
- `s4 map show` row table (short id, status, confidence, pairing, signatures).
- USIR/graph `qualified` names; correspondence `source_signature` / `target_signature`.
- E2E: confirm `--name add` then `s4 certify` Valid; extras cannot be confirmed.
