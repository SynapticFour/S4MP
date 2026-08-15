# Contributing to S4MP

This repository is maintained by **one person**. Pull requests are welcome; they are not the current default workflow. Until branch protection is enabled, commits may land directly on `main`. CI on `main` is the gate that matters.

## Before You Code

1. Read [Engineering Standards](docs/engineering/ENGINEERING_STANDARDS.md).
2. Read the README maturity banner — do not add commands that claim certification or semantic equivalence.
3. For the Java→Rust pipeline, see [Porting Workflow](docs/guides/PORTING_WORKFLOW.md).
4. New architectural choices: add an [ADR](docs/adr/README.md) in the same change.

## Development Setup

```bash
rustup toolchain install stable --component rustfmt clippy
make install-hooks
cargo build --workspace
cargo test --locked --workspace
```

## Pre-PR / pre-push Checklist

`make install-hooks` runs `scripts/hooks/ci-check.sh`, which mirrors CI:

```bash
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --no-deps
bash scripts/check-tiers.sh
cargo deny check   # if cargo-deny is installed
```

## Pull Requests

When you open a PR:

- Branch from `main`: `feature/<short-description>`
- One logical change
- Tests for behavior changes
- Do not describe parked crates (`s4-api`, `s4-ui`, `s4-planner`) as shipped

## License

By contributing, you agree that your contributions are licensed under MIT OR Apache-2.0, matching the project.
