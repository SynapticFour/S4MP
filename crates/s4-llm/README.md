# s4-llm

LLM-agnostic reasoning contracts.

## Responsibility

Defines request, context, policy, and proposal types for AI-assisted reasoning. **No LLM provider dependencies.** Implementations are interchangeable plugins.

## Public API

| Module | Purpose |
|--------|---------|
| `request` | `ReasonRequest`, `ReasonIntent` |
| `context` | `ContextBundle` |
| `policy` | `ReasonPolicy` |
| `proposal` | `Proposal`, `ProposalLifecycle`, `ProposedClaim`, `ModelMetadata` |
| `provider` | `LlmProvider` trait |
| `heuristic` | `HeuristicLlmProvider` (offline; always Proposed) |

## Tier

3 — Intelligence (interfaces + offline heuristic)
