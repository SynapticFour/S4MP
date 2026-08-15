# ADR-011: No auto-apply refactors in core

- **Status:** Accepted
- **Date:** 2026-07-13
- **Deciders:** Synaptic Four

## Context

Silent code rewrite destroys trust and certification replay.

## Decision

Core never applies refactors. Plans stay proposals. `s4-planner` is parked.

## Consequences

No transformation backend in v0.1.
