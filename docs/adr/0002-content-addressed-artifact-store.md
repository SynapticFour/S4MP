# ADR-002: Content-addressed artifact store

- **Status:** Accepted
- **Date:** 2026-07-13
- **Deciders:** Synaptic Four
- **Related:** ADR-012, ADR-014

## Context

Port maps and graphs must be replayable and cacheable.

## Decision

Primary artifacts are Blake3 content-addressed JSON envelopes. Secondary indexes (`write_at`) store **pointers** to content ids, not envelopes at a non-hash path.

## Consequences

Reproducible caches. Index files are not CAS objects; `read` follows pointers.
