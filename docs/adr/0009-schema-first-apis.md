# ADR-009: Schema-first APIs

- **Status:** Accepted
- **Date:** 2026-07-13
- **Deciders:** Synaptic Four
- **Related:** ADR-014

## Context

Multiple consumers (CLI, future HTTP, reports) need stable envelopes.

## Decision

Artifacts carry `schema_version`. v0.1 encoding is JSON (ADR-014). Unknown major versions are rejected.

## Consequences

No protobuf. No HTTP schema until `s4-api` is unparked.
