# ADR-006: Layered graph projections

- **Status:** Accepted
- **Date:** 2026-07-13
- **Deciders:** Synaptic Four

## Context

Different consumers need syntax, semantic, and requirements views.

## Decision

One artifact store; multiple typed graph projections.

## Consequences

v0.1 materializes a single in-memory semantic graph per source. Other layers are unspecified until implemented.
