# S4MP Implementation Roadmap

> **Shipped product:** heuristic Java↔Rust port-map CLI (`heuristic-map-v2`).
> Architecture specs under `docs/` remain a **target model**. Do not read this file as a promise that the knowledge platform exists.
> Status markers: **done** · **parked**

## Strategy

**Product (now):** a porter can register two trees, review heuristic pairs with ids, confirm them, and run coverage/policy checks. That is the claimed use.

**Not the product:** HTTP API, UI, planner, WASM plugins, networked LLM, S4QL, semantic equivalence, or “the port is certified correct.”

**Rule:** deepen the port-map loop before filling contract-only crates.

```text
P0 honesty/schema/e2e
  → P1 port-map review loop   ← shipped product
    → P2–P6 thin slices / satellites (done as code, not marketed as a platform)
      → parked: WASM, API, UI, marketplace
```

---

## Phase 0 — Product honesty & foundation hygiene

| Item | Status |
|------|--------|
| Maturity docs + CLI honesty | **done** |
| Real `s4 init` | **done** |
| Freeze USIR v0.1 JSON Schema | **done** |
| ADR: JSON encoding; in-process plugins | **done** (ADR-014, ADR-015) |
| Fixture e2e (no GATK clone) | **done** (`tests/fixtures/mini-port`, `make e2e-fixture`) |

**Exit:** CI green on fixtures; no command pretends to certify semantic equivalence. ✅

---

## Phase 1 — Port-map review loop (the product)

**Status:** **done**

1. Parser: Tree-sitter Java/Rust, signatures, qualified names, AST call edges — **done**
2. Correspondence v2: exclusive name + signature Jaccard; pairs stay `Diverged` — **done**
3. Review UX: `s4 map show`, confirm/reject by `--id` prefix or `--name`, English diff with `id=` — **done**
4. Certify: default policy `min_ported:1`; heuristic-only maps are `Invalid` — **done**
5. Optional live GATK HaplotypeCaller slice via Makefile — **optional**, not the CI path

**Exit:** `make e2e-fixture` covers unconfirmed → Invalid and confirm `--name add` → Valid. ✅

---

## Phase 2 — Pass pipeline & `s4 analyze`

**Status:** **done** (orchestration around the port loop)

- `Pass` / `PassPipeline` in `s4-analysis` — **done**
- `s4 analyze` = graph all sources → map → diff — **done**
- In-process `RecordingEventSink` — **done**
- CI tier-enforcement script (`scripts/check-tiers.sh`) — **done**

---

## Phase 3 — Graph engine & query

**Status:** **done** (filter query, graph diff, basic metrics)

- Durable graph artifacts; `GraphDiff` by `(kind,label)` — **done** (`s4 graph diff`)
- Minimal `s4 query` (`all` | `kind:*` | `label~*`) + optional `--metrics` — **done**
- Metrics pass (counts, fan-in/out via avg calls/callable) — **done**
- `LanguageId` → `s4-core` — **done** in Phase 6

---

## Phase 4 — Knowledge + requirements (thin / satellite)

**Status:** **done** (thin slice; hidden from `s4 --help`)

- SKG naming concept extractor — **done** (`s4 knowledge extract`)
- Requirements CRUD + OpenAPI path import — **done** (`s4 require …`)
- Trace suggest/apply by name — **done** (`s4 require trace-suggest`)
- Full curation UI / SOW Markdown parser — **parked**

---

## Phase 5 — Verification & certification

**Status:** **done** (honest thresholds, not semantic equivalence)

- `s4 verify` over map coverage + optional requirements traces — **done**
- `s4 certify` = policy over `VerificationRun` only — **done** (`default` requires ≥1 Ported row)
- Maturity remains `heuristic-map-v2`
- Showcase live certificate attachment — **parked**

---

## Phase 6 — Plugin host & LLM consumer

**Status:** **done** (in-process; WASM **parked** — ADR-016)

- `InProcessPluginHost` + built-in manifests; `s4 plugin list` — **done**
- `HeuristicLlmProvider` + `s4 reason` (outputs always `Proposed`) — **done**
- WASM sandbox / remote registry / third-party trust — **parked**

---

## Decision gates

| Decision | ADR | Notes |
|----------|-----|--------|
| Stay on JSON CAS (v0.1) | ADR-014 | Accepted |
| In-process plugins only (Phases 0–5) | ADR-015 | Accepted |
| Phase 6 host in-process; WASM later | ADR-016 | Accepted |
| What `s4 certify` Valid means | — | Policy over verification counters (`min_ported:1`). Not semantic Java↔Rust equivalence. |

## Non-goals (until an ADR says otherwise)

- Sonar-like rule catalog in core
- Neo4j / distributed graph
- Remote plugin marketplace
- Proof-assistant verification
- Replacing `gatk-rs` parity tests
- WASM plugin sandbox
- Auto-`Ported` on high confidence
- Marketing architecture specs as shipped features
