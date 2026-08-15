# ADR-008: Rust for core platform

- **Status:** Accepted
- **Date:** 2026-07-13
- **Deciders:** Synaptic Four

## Context

Need memory safety, performance, and a future WASM path.

## Decision

Core crates are Rust. `unsafe_code` is workspace-forbidden.

## Consequences

MSRV 1.75. Tree-sitter grammars are first-party dependencies, not subprocesses.
