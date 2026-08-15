# ADR-010: Query engine independent of AI

- **Status:** Accepted
- **Date:** 2026-07-13
- **Deciders:** Synaptic Four

## Context

Graph query must work with the network off.

## Decision

`s4 query` is a deterministic filter (`all` | `kind:*` | `label~*`). It does not call an LLM.

## Consequences

S4QL / Datalog remains an open decision. Do not advertise a query language beyond the filter subset.
