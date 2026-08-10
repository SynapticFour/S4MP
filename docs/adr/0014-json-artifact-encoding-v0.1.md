# ADR-014: JSON artifact encoding for v0.1

- **Status:** Accepted
- **Date:** 2026-08-10
- **Deciders:** SynapticFour architecture baseline
- **Related:** [ARCHITECTURE.md](../architecture/ARCHITECTURE.md) open decision #2, [IMPLEMENTATION_ROADMAP.md](../guides/IMPLEMENTATION_ROADMAP.md), [artifact.schema.json](../../schemas/artifact.schema.json)

## Context

Architecture left open whether artifact payloads use Protocol Buffers, JSON Schema + CBOR, or JSON. The working porting pipeline already persists JSON envelopes under `.s4/store/`. Changing encoding mid-slice would break fixtures, Showcase sidecars, and local caches without meaningful product gain.

## Decision

For **schema version 0.1**:

1. Artifact envelopes and USIR modules are **JSON** (pretty or compact), validated against JSON Schema under `schemas/`.
2. Content addressing continues to hash the **canonical serialized bytes** written to the store (as today).
3. A later major schema bump may introduce CBOR or Protobuf **with an explicit migration**, not as a silent dual format.

## Consequences

### Positive

- Matches shipped CLI behavior; no migration tax for Phase 0–1.
- Human-inspectable CAS artifacts aid debugging and Showcase honesty.
- JSON Schema documents freeze the interchange contract without codegen complexity.

### Negative

- Larger on-disk size and slower parse than binary encodings at scale.
- Must revisit before graph engines store tens of millions of nodes.

### Neutral

- Schema files remain the source of truth for field presence; Rust types must stay aligned.

## Alternatives Considered

| Alternative | Why rejected |
|-------------|--------------|
| Protobuf from day one | Breaks existing stores; premature for heuristic maps |
| CBOR + JSON Schema | Better density, but no current consumer needs it |
| Dual JSON/CBOR readers now | Doubles surface area before pass manager exists |

## Compliance

- New artifact kinds update `schemas/*.schema.json` in the same PR.
- CI fixture e2e asserts JSON manifests under `.s4/`.
