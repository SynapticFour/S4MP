# ADR-004: Plugins at volatile boundaries

- **Status:** Accepted
- **Date:** 2026-07-13
- **Deciders:** Synaptic Four
- **Related:** ADR-015, ADR-016

## Context

Languages and LLM vendors change faster than the core.

## Decision

Volatile behavior attaches via plugin contracts. Through Phase 6, first-party frontends run **in-process** and are dispatched by language id (`extract_for_language`). There is no third-party loader.

## Consequences

`s4 plugin list` must only list frontends the CLI actually uses. WASM remains deferred.
