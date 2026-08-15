# ADR-003: USIR as universal interchange

- **Status:** Accepted
- **Date:** 2026-07-13
- **Deciders:** Synaptic Four

## Context

Analyzers must not depend on Java or Rust syntax trees.

## Decision

Parsers lower to Universal Semantic IR (USIR). Analyzers consume USIR / graphs.

## Consequences

v0.1 USIR is a thin entity/relation subset (callables, types, heuristic calls). It is not LLVM IR.
