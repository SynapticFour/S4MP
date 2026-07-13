# Verification Engine
## Architecture Specification v0.1

> **Status:** Design baseline — no implementation  
> **Contract crate:** `s4-verification`  
> **Related:** [Requirements Graph](../requirements/REQUIREMENTS_GRAPH.md), [UCG](../graph/UNIVERSAL_CODE_GRAPH.md), [SKG](../knowledge/SOFTWARE_KNOWLEDGE_GRAPH.md), [Plugin System](../plugins/PLUGIN_SYSTEM.md), [Canonical Model](../model/CANONICAL_MODEL.md)

---

## 1. Purpose

The **Verification Engine** compares **two software versions** — typically a **baseline** (agreed, certified, or released) and a **candidate** (proposed upgrade, refactor, or fork) — and assesses whether **agreed requirements remain preserved**.

It does **not** produce a single boolean "safe to ship." It produces **evidence-backed judgments with confidence scores**, explicit gaps, and honest limits on what each technique can establish.

| Goal | Mechanism |
|------|-----------|
| Requirement preservation across versions | Baseline triple vs candidate triple + trace replay |
| Multi-signal verification | Static analysis, graph/API/schema/dependency diff, tests, property tests, LLM assist |
| Epistemic honesty | Confidence scores, verdict bands, limitation records |
| Certification input | Immutable `VerificationRun` artifacts for `s4-certification` |
| Replayability | All inputs pinned to CAS snapshot IDs |

**Core question:** Given agreed requirements **R** at baseline **A**, does candidate **B** still satisfy **R** — and with what confidence?

---

## 2. What This Engine Is Not

S4MP verification is **evidence aggregation**, not mathematical proof.

| We do **not** claim | Why |
|---------------------|-----|
| Full behavioral equivalence | Undecidable in general (Rice's theorem) |
| Absence of all bugs | Testing and analysis are incomplete |
| Semantic correctness of LLM judgments | LLM outputs are proposals, not facts |
| Runtime behavior from static graphs alone | Dynamic paths, I/O, concurrency, timing |
| Proof that unchanged code behaves identically | Environment, data, and deployment may differ |

When certainty is impossible, the engine reports **confidence and gaps** — never a false binary pass.

---

## 3. Position in the Platform

```
┌─────────────────────────────────────────────────────────────────┐
│  BASELINE (A)                    CANDIDATE (B)                   │
│  RequirementsSnapshot_A          RequirementsSnapshot_B          │
│  UcgSnapshot_A                   UcgSnapshot_B                   │
│  SkgSnapshot_A (optional)        SkgSnapshot_B (optional)        │
│  TestSuite_A (pinned)            TestSuite_B (executed)          │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    VERIFICATION ENGINE                           │
│  Orchestrator · Evidence collectors · Confidence aggregator      │
└───────┬─────────────────┬─────────────────┬───────────────────┘
        │                 │                 │
        ▼                 ▼                 ▼
  StaticAnalysis    GraphDiffSuite    TestExecution
  ApiComparator     SchemaComparator  PropertyTesting
  DepComparator     LlmSemanticAssist
        │                 │                 │
        └─────────────────┴─────────────────┘
                             │
                             ▼
              VerificationRun artifact (per comparison)
                             │
                             ▼
                    s4-certification (optional gate)
```

| Layer | Role |
|-------|------|
| **Requirements Graph** | Defines *what* must be preserved (`must` requirements, baselines) |
| **UCG / SKG** | Structural and semantic context for diff and trace replay |
| **Verification Engine** | Collects evidence; scores preservation per requirement |
| **Certification** | Policy over verification runs (thresholds, waivers, audit) |

---

## 4. Core Operation: Version Comparison

### 4.1 Comparison Request

```
VersionComparisonRequest {
  comparison_id:     ComparisonId
  baseline: {
    requirements:    RequirementsSnapshotId
    code:            UcgSnapshotId
    knowledge:       Option<KnowledgeSnapshotId>
    test_suite:      Option<TestSuiteRef>      // pinned artifact or manifest
    openapi:         Option<ArtifactId[]>
    schemas:         Option<ArtifactId[]>
  }
  candidate: { /* same shape */ }
  scope: {
    requirement_ids: RequirementId[] | "all_active_must"
    include_should:  bool
    include_may:     bool
  }
  policy:            VerificationPolicyRef     // weights, thresholds, enabled checks
}
```

### 4.2 Comparison Output

```
VersionComparisonResult {
  comparison_id:     ComparisonId
  overall_verdict:   PreservationVerdict
  overall_confidence: f32                       // 0.0–1.0
  requirement_results: RequirementPreservationResult[]
  evidence_bundle:   ArtifactId                // all sub-results
  gaps:              VerificationGap[]         // what we could not check
  limitations:       LimitationRecord[]        // epistemic bounds applied
  artifact:          ArtifactId
}
```

### 4.3 Preservation Verdict Bands

Avoid binary pass/fail at the top level. Use **verdict bands**:

| Verdict | Meaning | Typical confidence |
|---------|---------|------------------|
| **verified** | Strong evidence; no material gaps for scoped requirements | ≥ 0.95 |
| **likely_preserved** | Evidence supports preservation; minor gaps or low-risk unknowns | 0.75–0.94 |
| **inconclusive** | Insufficient or conflicting evidence | 0.40–0.74 |
| **likely_violated** | Evidence suggests regression; not all signals agree | 0.20–0.39 |
| **violated** | Deterministic or high-confidence failure | < 0.20 (inverted: violation confidence high) |
| **not_applicable** | Requirement out of scope, waived, or untraceable | — |

**Certification policies** may map bands to gate decisions (e.g. block release on `violated` or `inconclusive` for SEC-* requirements).

---

## 5. Verification Dimensions

Each dimension is a **plugin-backed evidence collector**. The orchestrator runs enabled collectors in parallel (where independent) and merges results.

### 5.1 Static Analysis

**What it checks:** Lint rules, type errors, security scanners, architecture rule violations, nullability, taint flows.

| Input | Baseline A vs candidate B source trees or IR artifacts |
| Output | `StaticAnalysisEvidence { findings[], delta: new|resolved|unchanged }` |

| Strength | Weakness |
|----------|----------|
| Fast, deterministic for configured rules | Rule coverage is finite; false positives/negatives |
| Good for SEC-*, ARCH-* constraints | Cannot prove runtime behavior |
| High confidence when rule fires on regression | Silence ≠ proof of safety |

**Confidence model:**
- New **error**-severity finding on candidate trace target → violation confidence 0.85–0.95
- New **warning** → 0.5–0.7 (inconclusive band)
- Resolved finding → positive signal (+0.05 to preservation confidence for linked requirement)

**Limitation record:** `"Static analysis is sound only for encoded rules; unmodeled behavior is unchecked."`

---

### 5.2 Graph Comparison (UCG + SKG)

**What it checks:** Structural and semantic graph diffs between snapshots.

**UCG diff categories:**

| Change class | Preservation impact |
|--------------|---------------------|
| Symbol removed | High risk if traced to requirement |
| Callable signature changed | API/behavior risk |
| Call edge added/removed | Flow change |
| Dependency edge changed | Integration risk |
| Architectural boundary violation | ARCH constraint risk |

**SKG diff (optional):** Concept removed, `realizes` edge broken, domain membership change.

```
GraphComparisonEvidence {
  ucg_diff:       UcgDiffArtifact,
  skg_diff:       Option<SkgDiffArtifact>,
  trace_impact:   TraceImpactReport[]   // per requirement: broken | weakened | unchanged | strengthened
}
```

| Strength | Weakness |
|----------|----------|
| Deterministic structural diff | Refactor with identical behavior looks like large diff |
| Trace replay: requirement → symbol still exists? | Renames need identity resolution (heuristic or manual) |
| Architectural constraint checking | Cannot see dynamic dispatch, reflection, plugins |

**Confidence model:**
- Traced symbol **removed** with no replacement mapping → 0.9 violation confidence
- Signature change on traced callable → 0.7 inconclusive (needs API/test confirmation)
- Diff only in untraced modules → 0.95 preserved (for that requirement)

**Identity resolution:** Compare symbols via stable IDs where available; fall back to qualified name + signature hash; LLM assist for rename detection (proposed, low confidence).

**Limitation record:** `"Graph comparison establishes structural delta, not behavioral equivalence."`

---

### 5.3 API Comparison

**What it checks:** Public interface compatibility between baseline and candidate.

**Sources:** OpenAPI, Protobuf, GraphQL SDL, language-exported API surfaces (via parser plugins).

```
ApiComparisonEvidence {
  contract_ref:    RequirementId | ArtifactId,
  breaking_changes: BreakingChange[],
  additive_changes: AdditiveChange[],
  compatibility:   backward_compatible | breaking | unknown
}
```

| Check | Deterministic? |
|-------|----------------|
| Endpoint removed | Yes → breaking |
| Required field added to request | Yes → breaking |
| Optional response field removed | Yes → breaking |
| Internal implementation change | Out of scope for API diff |

**Confidence model:**
- Deterministic breaking change on traced ApiContract → 0.95–1.0 violation
- Backward compatible per schema rules → 0.9 preserved for API requirement
- Undocumented or partial spec → cap at 0.7; gap recorded

**Limitation record:** `"API compatibility does not guarantee semantic compatibility of implementations."`

---

### 5.4 Schema Comparison

**What it checks:** Data contract evolution — JSON Schema, SQL migrations, Avro, Protobuf messages, event payloads.

```
SchemaComparisonEvidence {
  schema_pair:     (ArtifactId, ArtifactId),
  diff_kind:       compatible | breaking | incompatible | unparseable,
  migration_present: bool,
  details:         SchemaDiffEntry[]
}
```

| Rule (example) | Verdict |
|----------------|---------|
| Field removed without default | Breaking |
| Enum value removed | Breaking |
| Widening numeric type | Often compatible (policy-dependent) |
| Migration script absent for breaking DB change | Violation + gap |

**Confidence:** High (0.9+) when formal schema diff succeeds; lower when schemas are informal or embedded in code.

**Limitation record:** `"Schema diff validates structure, not data quality, migration correctness at runtime, or backfill completeness."`

---

### 5.5 Dependency Comparison

**What it checks:** Third-party and internal module dependency changes.

```
DependencyComparisonEvidence {
  added:           DependencyRef[],
  removed:         DependencyRef[],
  version_changes: VersionChange[],
  license_changes: LicenseChange[],
  vulnerability_delta: VulnFinding[]
}
```

| Signal | Requirement link |
|--------|------------------|
| New CVE on upgraded lib | SEC-* |
| License change GPL → proprietary | SEC / legal constraint |
| Major version bump of traced lib | inconclusive until tests run |
| Removed dependency still referenced | 0.95 violation (linker/static proof) |

**Confidence:** Deterministic for lockfile/manifest diff; vulnerability data depends on DB freshness (timestamp + gap).

**Limitation record:** `"Dependency comparison does not analyze transitive runtime behavior or supply-chain integrity beyond configured scanners."`

---

### 5.6 Test Execution

**What it checks:** Regression suite against candidate build.

```
TestExecutionEvidence {
  suite_ref:       TestSuiteRef,
  baseline_result: Option<TestRunSummary>,   // historical if available
  candidate_result: TestRunSummary,
  requirement_mapping: TestRequirementMap[]  // test → requirement
}

TestRunSummary {
  passed, failed, skipped, flaky,
  coverage_delta:  Option<CoverageDelta>
}
```

| Outcome | Preservation signal |
|---------|---------------------|
| All mapped tests pass | Strong positive |
| Previously passing test fails | Strong negative |
| New tests pass (not in baseline) | Weak positive only |
| Skipped / flaky | Gap recorded |

**Confidence model:**
- Fail on test traced to AC-* → 0.85–0.95 violation for parent BR
- Pass on full mapped suite → 0.8–0.9 preserved (coverage limits apply)
- No mapped tests for requirement → gap; cap requirement confidence at 0.6

**Limitation record:** `"Tests demonstrate behavior only for exercised inputs; passing tests do not prove correctness."`

---

### 5.7 Property-Based Testing

**What it checks:** Invariants over generated inputs — algebraic laws, round-trips, idempotence, monotonicity.

```
PropertyTestEvidence {
  properties: PropertyResult[],
  iterations: u32,
  seed: u64,
  shrunk_counterexample: Option<ArtifactId>
}

PropertyResult {
  name: string,
  requirement_ref: Option<RequirementId>,
  status: pass | fail | inconclusive,
  samples_tested: u32
}
```

| Strength | Weakness |
|----------|----------|
| Explores input space beyond fixed examples | Finite samples — not proof for infinite domains |
| Strong for PERF-* (bounded checks) and data invariants | Property authorship quality varies |
| Counterexample is concrete evidence | Flaky properties, environment sensitivity |

**Confidence model:**
- Property fail with reproducible seed → 0.9 violation for linked requirement
- Property pass after N iterations → 0.7–0.85 preserved (state sample count in evidence)
- Property not run → gap

**Limitation record:** `"Property-based tests sample a finite subset of inputs; they do not constitute formal verification unless paired with a proof assistant (out of scope v0.1)."`

---

### 5.8 LLM-Assisted Semantic Comparison

**What it checks:** Semantic similarity of changed regions, docstrings, requirement statements vs code behavior descriptions, rename/refactor equivalence hypotheses.

```
LlmSemanticEvidence {
  comparisons: SemanticComparison[],
  model_id: string,
  prompt_hash: ArtifactId,
  lifecycle: proposed                    // always until human/verifier accepts
}

SemanticComparison {
  scope:           SymbolPair | ModulePair | RequirementStatementPair,
  judgment:        equivalent | likely_equivalent | divergent | unknown,
  confidence:      f32,                  // model self-score — capped by platform
  rationale:       string,
  lifecycle:       proposed | accepted | rejected
}
```

| Use case | Platform cap on confidence |
|----------|---------------------------|
| "Is this refactor behavior-preserving?" | **Max 0.75** in aggregation |
| "Does this code implement this requirement text?" | **Max 0.70** |
| Rename detection assist | **Max 0.65** unless confirmed by graph ID |

**Rules:**
1. LLM evidence **never alone** satisfies a `must` requirement for certification
2. Must be corroborated by test, API diff, or human acceptance to enter aggregation above cap
3. Always stored with `lifecycle: proposed` until accepted
4. Failures in LLM call → gap, not pass

**Limitation record:** `"LLM comparison is heuristic, non-deterministic, and may hallucinate; it is advisory evidence only."`

---

## 6. Requirement Preservation Logic

For each requirement **r** in scope:

```
RequirementPreservationResult {
  requirement_id:    RequirementId
  verdict:           PreservationVerdict
  confidence:        f32
  evidence_refs:     ArtifactId[]          // links to dimension evidence
  trace_status:      intact | broken | stale | waived
  contributing_signals: SignalContribution[]
  gaps:              VerificationGap[]
}
```

### 6.1 Evaluation Order

1. **Waived** → `not_applicable` (record waiver artifact)
2. **Trace broken** (symbol removed, no mapping) → start at `likely_violated` unless counter-evidence
3. **Collect dimension evidence** linked to **r** or child AC-*
4. **Aggregate confidence** (§7)
5. **Map to verdict band**

### 6.2 Rollup (Parent Requirements)

Business requirements roll up from acceptance criteria and child constraints:

```
parent.confidence = weighted_min(child.confidences, weights by priority)
parent.verdict    = worst_child_verdict_band (must-children only)
```

A single `violated` must-child pulls parent to at least `likely_violated`.

### 6.3 Trace Replay

When comparing A → B:

```
for link in traces(r, snapshot_A):
  resolve link.target in UCG_B
  if missing → trace_status = broken
  if changed signature/API → trace_status = stale
  else → trace_status = intact
```

Trace status modulates confidence ceilings (broken trace caps at 0.5 until relinked).

---

## 7. Confidence Aggregation

### 7.1 Signal Weights (Default Policy)

| Dimension | Default weight | Max solo confidence |
|-----------|----------------|---------------------|
| Test execution (mapped) | 0.30 | 0.90 |
| API comparison | 0.20 | 0.95 |
| Schema comparison | 0.15 | 0.95 |
| Graph comparison (trace) | 0.15 | 0.90 |
| Static analysis | 0.10 | 0.90 |
| Dependency comparison | 0.05 | 0.85 |
| Property-based testing | 0.05 | 0.85 |
| LLM semantic | 0.05 | **0.75 (hard cap)** |

Weights are policy-configurable per project and requirement kind (e.g. SEC-* boosts static analysis weight).

### 7.2 Aggregation Formula (v0.1)

Simple **weighted evidence combination with conflict penalty**:

```
preservation_score = Σ (weight_i × signal_i) / Σ weight_i
  where signal_i ∈ [0, 1] (1 = fully preserved, 0 = clearly violated)

if ∃ high_confidence_violation (signal < 0.3, source confidence > 0.85):
  preservation_score = min(preservation_score, 0.35)

if ∃ unresolved_gap for must requirement:
  preservation_score = min(preservation_score, 0.74)   // cannot reach "verified"

verdict = band_map(preservation_score)
```

**Conflicting signals** (e.g. tests pass but API breaking) → `inconclusive` + both evidences attached; human review queued.

### 7.3 Uncertainty Propagation

Record explicitly:

```
VerificationGap {
  requirement_id: Option<RequirementId>,
  dimension:      string,
  reason:         string,    // "no mapped tests", "openapi missing", "llm timeout"
  impact:         caps_confidence_at: f32
}
```

Gaps **lower ceilings**; they never silently increase confidence.

---

## 8. Orchestration

### 8.1 Verification Pipeline

```
VersionComparisonRequest
  → validate snapshots exist in CAS
  → load RequirementsBaseline + active must set
  → plan evidence collectors (policy-driven DAG)
  → run collectors (parallel where safe)
  → merge evidence bundle
  → per-requirement preservation scoring
  → emit VerificationRun artifact
  → optional: trigger CertificationPolicy evaluation
```

### 8.2 Plugin Traits

Core orchestrator lives in `s4-verification`; collectors are plugins:

```rust
trait VersionComparator: Plugin {
  fn compare(&self, ctx: &mut InvokeContext, req: VersionComparisonRequest)
    -> Result<VersionComparisonResult>;
}

trait EvidenceCollector: Plugin {
  fn dimension(&self) -> EvidenceDimension;
  fn collect(&self, ctx: &mut InvokeContext, req: ComparisonContext)
    -> Result<EvidenceArtifact>;
}

trait Verifier: Plugin {
  fn verify(&self, ctx: &mut InvokeContext, request: VerifyRequest) -> Result<()>;
}
```

`VerifyRequest` covers single-shot checks (trace completeness, invariant sets) as well as full version comparisons.

### 8.3 Determinism

| Component | Deterministic? |
|-----------|----------------|
| Graph diff, API diff, schema diff, lockfile diff | Yes (given same inputs) |
| Static analysis (fixed rules/version) | Yes |
| Test execution | Mostly (flaky tests flagged) |
| Property tests (fixed seed) | Reproducible |
| LLM semantic | **No** — temperature, model version logged |

Verification runs pin **all input artifact IDs** for replay.

---

## 9. Artifacts and Persistence

### 9.1 VerificationRun

```
VerificationRun {
  id:                 VerificationId
  kind:               version_comparison | invariant_check | trace_completeness
  baseline_refs:      SnapshotRefs
  candidate_refs:     SnapshotRefs
  policy_ref:         ArtifactId
  overall_verdict:    PreservationVerdict
  overall_confidence: f32
  requirement_results: ArtifactId      // shard
  evidence_bundle:    ArtifactId
  gaps:               VerificationGap[]
  limitations:        LimitationRecord[]
  started_at, completed_at,
  orchestrator_version: string
}
```

Stored immutably in CAS. Certificates reference `VerificationRun` artifact IDs.

### 9.2 Evidence Bundle Structure

```
EvidenceBundle {
  static_analysis:    Option<ArtifactId>,
  graph_comparison:   Option<ArtifactId>,
  api_comparison:     ArtifactId[],
  schema_comparison:  ArtifactId[],
  dependency_comparison: Option<ArtifactId>,
  test_execution:     Option<ArtifactId>,
  property_testing:   Option<ArtifactId>,
  llm_semantic:       Option<ArtifactId>,
}
```

---

## 10. Integration with Certification

`s4-certification` consumes verification output; it does **not** re-run checks.

```
CertificationPolicy {
  rules: [
    { require: "no requirement with verdict violated" },
    { require: "SEC-* confidence >= 0.85" },
    { require: "overall_confidence >= 0.80" },
    { allow_waiver: true, waiver_artifact_required: true }
  ]
}
```

| Verification output | Certificate effect |
|--------------------|--------------------|
| All rules pass | `Valid` |
| Rule fail, no waiver | `Invalid` |
| Inconclusive on must SEC | Policy-defined (often `Invalid`) |

Human sign-off on `inconclusive` may produce waiver artifact → policy may allow `Valid` with audit trail.

---

## 11. Honest Limitations (Summary)

The engine documents these limits **in every comparison result**, not only in documentation.

### 11.1 Fundamental Limits

| Limit | Implication |
|-------|-------------|
| **Behavioral equivalence is undecidable** | No general proof that B behaves like A |
| **Incomplete test coverage** | Untested paths may regress silently |
| **Environment and deployment drift** | Verification pins artifacts, not production config |
| **Nondeterminism** | Threads, timing, external services, ML models |
| **Malicious code** | Verification assumes good-faith codebase; not a security audit substitute |

### 11.2 Technique-Specific Limits

| Technique | Cannot establish |
|-----------|------------------|
| Static analysis | Runtime-only failures, unmodeled rules |
| Graph comparison | Behavioral sameness after refactor |
| API / schema diff | Implementation correctness behind interface |
| Dependency diff | Vulnerabilities in unaudited code paths |
| Test execution | Coverage outside suite |
| Property testing | Universal quantification |
| LLM semantic | Ground truth; reproducibility across models |

### 11.3 Requirement Trace Limits

- Stale or missing traces reduce confidence — they do not imply preservation
- Waivers are explicit non-checks, not passes
- Requirements text ambiguity remains human-resolvable

### 11.4 Language in Reports

**Use:**
- "Evidence supports preservation (confidence 0.82)"
- "Deterministic API breaking change detected"
- "Inconclusive: no mapped regression tests"

**Avoid:**
- "Proven safe"
- "Mathematically equivalent"
- "Guaranteed requirement satisfaction"
- "AI confirms compliance" (without corroboration)

---

## 12. Example Comparison Flow

**Scenario:** Upgrade baseline `v1.0` → candidate `v1.1` for Order Management SOW.

| Step | Result |
|------|--------|
| Load baseline triple (Req + UCG + SKG @ v1.0) | 12 active must requirements |
| Graph diff | `OrderService.create` signature changed → trace stale for API-001 |
| API diff (OpenAPI) | Breaking: response field `legacyId` removed → violation confidence 0.97 |
| Schema diff | DB migration present; compatible → 0.92 |
| Tests | 847/847 pass → 0.88 |
| Property test | `order_roundtrip` pass (1000 iter) → 0.82 |
| LLM | "Refactor appears equivalent" → 0.65 (proposed, not in aggregate above cap) |
| **API-001** | **violated** (0.97) |
| **BR-001** (parent) | **likely_violated** (rollup) |
| **Overall** | **likely_violated** (0.38) — gaps: none for API-001 |

Certification policy blocks release until API restored or requirement superseded with waiver.

---

## 13. Phased Delivery

| Phase | Deliverable |
|-------|-------------|
| **V0** | `VersionComparisonRequest/Result` schema; verdict bands; gap model |
| **V1** | Graph diff + trace replay; evidence bundle |
| **V2** | API + schema comparators (OpenAPI, JSON Schema) |
| **V3** | Test execution integration + requirement mapping |
| **V4** | Static analysis + dependency diff collectors |
| **V5** | Property test harness integration |
| **V6** | Confidence aggregator + certification policy hooks |
| **V7** | LLM semantic assist (proposed-only, capped) |

---

## 14. Open Decisions

1. **Conflict resolution UI** — merge conflicting signals manually vs auto-prefer deterministic
2. **Symbol identity** — global stable IDs vs per-snapshot hash (rename problem)
3. **Flaky test policy** — quarantine vs fail comparison
4. **Formal methods hook** — optional proof assistant evidence type (future)
5. **Continuous vs on-demand** — verify every commit vs release gate only

---

## 15. Summary

The Verification Engine:

1. **Compares two software versions** against an agreed requirements baseline
2. **Combines eight evidence dimensions** — static analysis, graph, API, schema, dependency, tests, property tests, LLM assist
3. **Returns confidence-scored verdict bands**, not false binary certainty
4. **Records gaps and limitations** in every run
5. **Never claims mathematical proof** of equivalence or full correctness
6. **Feeds certification** with immutable, replayable `VerificationRun` artifacts

It closes the loop from **what was promised** (Requirements Graph) to **what changed** (UCG/SKG diffs) to **what we can evidence** (tests, schemas, analysis) — with epistemic honesty about everything we cannot.
