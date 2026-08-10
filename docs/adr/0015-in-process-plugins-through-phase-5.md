# ADR-015: In-process plugins through Phase 5

- **Status:** Accepted
- **Date:** 2026-08-10
- **Deciders:** SynapticFour architecture baseline
- **Related:** [ARCHITECTURE.md](../architecture/ARCHITECTURE.md) open decision #1, [PLUGIN_SYSTEM.md](../plugins/PLUGIN_SYSTEM.md), [ADR-013](./0013-llvm-infrastructure-not-sonarqube.md), [IMPLEMENTATION_ROADMAP.md](../guides/IMPLEMENTATION_ROADMAP.md)

## Context

Plugin docs describe Phase 1 static in-process registration and later WASM sandboxing. Architecture listed “in-process vs WASM from day one” as open. Shipping WASM host, WIT, and signing before a trustworthy porting pipeline and pass manager would delay the only working product path.

## Decision

1. **Phases 0–5** use **trusted first-party, in-process** plugins (Rust crates linked into `s4-cli` / future engines).
2. Parser, analyzer, and verifier “plugins” may start as ordinary modules implementing `s4-plugin` traits — no dynamic loading required yet.
3. **WASM sandbox, remote registry, and third-party trust tiers are deferred** past Phase 5 (see [ADR-016](./0016-phase6-in-process-host-wasm-deferred.md) — Phase 6 ships an in-process host; WASM remains deferred).
4. Core must still avoid embedding language-specific rule catalogs (ADR-013); in-process does not mean “logic in `s4-core`.”

## Consequences

### Positive

- Unblocks Phase 1–5 without ABI/sandbox design thrash.
- Matches current Java/Rust tree-sitter frontends compiled into the workspace.
- Keeps debugging simple (native stack traces).

### Negative

- Third-party / untrusted plugins are unavailable until Phase 6.
- Switching to WASM later may require a thin adapter over existing traits.

### Neutral

- Static registration can evolve into a host registry without changing USIR or CAS contracts.

## Alternatives Considered

| Alternative | Why rejected |
|-------------|--------------|
| WASM from day one | Blocks porting depth and pass manager |
| Native `dlopen` plugins now | ABI stability cost without ecosystem demand |
| Keep forever in-process only | Rejects long-term sandbox goals; revisit at Phase 6 |

## Compliance

- No WASM runtime dependency in Tier 0–4 crates until an ADR supersedes this one.
- New language frontends land as workspace crates implementing parser traits.
