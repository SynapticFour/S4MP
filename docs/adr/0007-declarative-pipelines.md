# ADR-007: Declarative pipelines

- **Status:** Accepted
- **Date:** 2026-07-13
- **Deciders:** Synaptic Four

## Context

Ad-hoc CLI sequencing is hard to replay.

## Decision

The intended model is a pass pipeline over CAS artifacts. v0.1 `s4 analyze` is a fixed sequence (graph → map → diff) with an in-process event sink.

## Consequences

There is no Kubernetes-style job scheduler. Do not document one as shipped.
