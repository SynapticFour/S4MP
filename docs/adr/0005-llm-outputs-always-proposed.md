# ADR-005: LLM outputs always proposed

- **Status:** Accepted
- **Date:** 2026-07-13
- **Deciders:** Synaptic Four

## Context

Model output must not silently become ground truth.

## Decision

All LLM and heuristic reasoner outputs use lifecycle `Proposed`. Correspondence heuristics emit `Diverged`, never auto-`Ported`. Naming concepts are `Proposed` with confidence 0.4.

## Consequences

Default certification cannot treat heuristic rows as ported (`min_ported:1`).
