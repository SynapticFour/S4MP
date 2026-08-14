# s4-verification

Verification, invariant checking, and acceptance workflow contracts.

## Public API

| Module | Purpose |
|--------|---------|
| `port_diff` | **Live:** `VerificationRun`, `build_verification_run` (coverage/trace thresholds, not semantic equivalence) |
| `invariant` | `Invariant`, `InvariantSet` (contract) |
| `verifier` | `Verifier`, `VerificationResult` (contract; unused by CLI) |
| `workflow` | `AcceptanceWorkflow` trait (contract) |

## Tier

3 — Quality
