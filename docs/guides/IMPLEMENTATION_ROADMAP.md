# S4MP Implementation Roadmap

> Derived from architecture specs, ADR open questions, and the working v0.1 porting slice.
> Status markers: **done** · **in progress** · **next** · planned

## Strategy

**Near-term product:** trustworthy Java↔Rust port evidence (gatk-rs / Showcase).
**Long-term product:** knowledge platform (graphs → verify → certify).
**Rule:** deepen the working vertical slice before filling contract-only crates.

```text
P0 honesty/schema/e2e
  → P1 porting depth
    → P2 pass manager + analyze
      → P3 graph query + metrics
        → P4 SKG + requirements + traces
          → P5 verify + certify
            → P6 plugins + LLM + (later) WASM
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

**Exit:** CI green on fixtures; no command pretends to certify. ✅

---

## Phase 1 — Make the porting slice trustworthy

**Status:** **done** (signatures, cross-module calls, correspondence v2, JSON sidecar, parallel extract)

1. Parser: cross-file calls, signatures in USIR, parallel extract + sequential CAS write — **done**
2. Correspondence v2: name + signature Jaccard — **done**
3. Diff report v2: confidence bands + JSON sidecar — **done**
4. Optional live Showcase path with offline/cached GATK slice — deferred (fixtures remain CI path)

**Exit:** Repeatable live fixture `make e2e-fixture`; maturity `heuristic-map-v2` without certification claims. ✅

---

## Phase 2 — Pass pipeline & `s4 analyze`

**Status:** **done**

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
- `LanguageId` → `s4-core` (clears former tier exception) — **done** in Phase 6

---

## Phase 4 — Knowledge + requirements (thin)

**Status:** **done** (thin slice)

- SKG naming concept extractor — **done** (`s4 knowledge extract`)
- Requirements CRUD + OpenAPI path import — **done** (`s4 require …`)
- Trace suggest/apply by name — **done** (`s4 require trace-suggest`)
- Full curation UI / SOW Markdown parser — deferred

---

## Phase 5 — Verification & certification

**Status:** **done** (thin, honest thresholds)

- `s4 verify` over baseline/candidate map + optional requirements traces — **done**
- `s4 certify` = policy over `VerificationRun` only — **done** (`default` policy)
- Explicit honesty: not semantic equivalence; maturity remains `heuristic-map-v2`
- Showcase live certificate attachment — deferred

---

## Phase 6 — Plugin runtime & LLM consumer

**Status:** **done** (in-process; WASM deferred — ADR-016)

- Relocate `LanguageId` to `s4-core`; remove `s4-project → s4-parser` tier exception — **done**
- `InProcessPluginHost` + built-in manifests; `s4 plugin list` — **done**
- `HeuristicLlmProvider` + `s4 reason` (outputs always `Proposed`) — **done**
- WASM sandbox / remote registry / third-party trust — **deferred** (Phase 7+)

---

## Decision gates

| Decision | ADR | Before |
|----------|-----|--------|
| Stay on JSON CAS (v0.1) | ADR-014 | Phase 2 |
| In-process plugins only (Phases 0–5) | ADR-015 | Phase 2 |
| Phase 6 host in-process; WASM later | ADR-016 | Phase 6 |
| Symbol identity across renames | TBD | Phase 3–5 |
| What “certified” means for HC | TBD | Phase 5 |

## Non-goals (next ~6 months)

- Sonar-like rule catalog in core
- Neo4j / distributed graph
- Remote plugin marketplace
- Proof-assistant verification
- Replacing `gatk-rs` parity tests
- WASM plugin sandbox (until an ADR supersedes ADR-016)
