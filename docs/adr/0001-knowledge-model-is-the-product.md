# ADR-001: Knowledge model is the product

- **Status:** Accepted
- **Date:** 2026-07-13
- **Deciders:** Synaptic Four
- **Related:** [ARCHITECTURE.md](../architecture/ARCHITECTURE.md)

## Context

LLM vendors and IDE features churn. A platform that *is* an AI wrapper dies with the vendor.

## Decision

The durable product is the knowledge model (graphs, provenance, time). AI is one consumer among many.

## Consequences

Shipped v0.1 is a heuristic port map, not the full model. Specs must not be sold as implemented features.

## Alternatives Considered

| Alternative | Why rejected |
|-------------|--------------|
| LLM-first product | Vendor lock-in |
