# Contributing to S4MP

Thank you for contributing. All work must follow the platform engineering standards.

## Before You Code

1. Read [Engineering Standards](docs/engineering/ENGINEERING_STANDARDS.md) — **mandatory** for every PR.
2. Read [Architecture Specification](docs/architecture/ARCHITECTURE.md) for system context.
3. For architectural choices, open an [ADR](docs/adr/README.md) or [RFC](docs/rfc/README.md) before large implementations.

## Development Setup

```bash
rustup toolchain install stable --component rustfmt clippy
cargo build --workspace
cargo test --workspace
```

## Pre-PR Checklist

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
```

Install [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) and run `cargo deny check` if you changed dependencies.

See [Engineering Standards §21](docs/engineering/ENGINEERING_STANDARDS.md#21-pre-implementation-checklist) for the full list.

## Pull Requests

- Branch from `main`: `feature/<issue>-<short-description>`
- Link an issue or ADR/RFC
- One logical change per PR when practical
- Documentation and tests with behavior changes

## License

By contributing, you agree that your contributions are licensed under MIT OR Apache-2.0, matching the project.
