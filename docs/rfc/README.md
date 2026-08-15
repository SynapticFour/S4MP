# Request for Comments (RFC)

RFCs propose **cross-cutting changes** that affect multiple crates, public plugin APIs, CI policy, or contributor workflow. They complement [ADRs](../adr/README.md), which record single architectural decisions.

**Staffing:** this repository has one maintainer. The process below is the intended bar, not a staffed review roster. A 5-day comment period is **not** currently enforced. Prefer a short ADR in the same change unless the change is truly cross-cutting.

## When to Write an RFC

| Write an RFC | Write an ADR instead |
|--------------|----------------------|
| New S4QL query language | Storage engine choice |
| Breaking plugin SDK change | Blake3 for artifact IDs |
| Workspace-wide MSRV bump policy | In-process vs WASM phase 1 |
| Performance budget revision | |

See [Engineering Standards §12](../engineering/ENGINEERING_STANDARDS.md#12-rfc-process).

## Process

1. **Draft** — copy [`0000-template.md`](./0000-template.md) to `NNNN-short-title.md`; open PR with status `Draft`.
2. **Review** — minimum **5 business days** comment period for breaking or public API changes.
3. **Revise** — address feedback; link spawned ADRs for sub-decisions.
4. **Accept** — merge with status `Accepted`; create tracking issue for implementation.
5. **Implement** — work proceeds only after RFC is `Accepted` (or scoped ADR for subset).

Statuses: `Draft` → `Review` → `Accepted` | `Rejected` | `Withdrawn`

## Index

| RFC | Title | Status |
|-----|-------|--------|
| — | — | — |
