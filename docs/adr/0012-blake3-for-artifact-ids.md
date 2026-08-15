# ADR-012: Blake3 for artifact IDs

- **Status:** Accepted
- **Date:** 2026-07-13
- **Deciders:** Synaptic Four
- **Related:** ADR-002

## Context

Git SHA-1 is the wrong hash for new CAS.

## Decision

`ArtifactId` is Blake3 of canonical compact JSON bytes.

## Consequences

IDs are not git-compatible. Indexes must not pretend a non-hash path is a content address (pointers, ADR-002).
