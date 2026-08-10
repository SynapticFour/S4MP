# ADR-016: Phase 6 in-process plugin host; WASM still deferred

- **Status:** Accepted
- **Date:** 2026-08-10
- **Deciders:** SynapticFour architecture baseline
- **Related:** [ADR-015](./0015-in-process-plugins-through-phase-5.md), [PLUGIN_SYSTEM.md](../plugins/PLUGIN_SYSTEM.md), [IMPLEMENTATION_ROADMAP.md](../guides/IMPLEMENTATION_ROADMAP.md)

## Context

ADR-015 deferred WASM until Phase 6. Phase 6 needs a real plugin **host** and an LLM consumer path without blocking on sandbox/WIT/signing design. First-party parsers and the heuristic reasoner already live in the workspace binary.

## Decision

1. Ship **`InProcessPluginHost`** with static registration of built-in manifests (`s4 plugin list`).
2. Ship **`HeuristicLlmProvider`** (offline) exposed as `s4 reason`; all outputs use **`ProposalLifecycle::Proposed`** only (ADR-005).
3. Relocate **`LanguageId`** to `s4-core` so tier-1 `s4-project` no longer depends on tier-2 `s4-parser`.
4. **WASM sandbox, remote registry, and third-party trust tiers remain deferred** (Phase 7+ / superseding ADR). No WASM runtime in Tier 0–4 crates.

## Consequences

### Positive

- CLI can enumerate plugins and produce honest Proposed proposals without network.
- Dependency tiers are clean (no `s4-project → s4-parser` exception).

### Negative

- Untrusted plugins still unavailable.
- Heuristic reasoner is not model-backed; networked providers remain future plugins.

## Compliance

- `scripts/check-tiers.sh` must pass with zero exceptions.
- LLM / reasoner APIs must not offer a path to emit Accepted facts.
