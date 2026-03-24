# `apply_splice` Governance Review

## Scope

This note reviews the two governance-facing drafts:

- `schedule_plan_draft.md`
- `acceptance_criteria_draft.md`

The goal is not to restate them.
The goal is to confirm that schedule, review gates, acceptance logic, and
definition-of-done are mutually consistent enough to guide the next phase.

## Consistency Result

The two drafts are materially aligned and can serve as the current governance
baseline for `apply_splice` work.
Stable product-boundary decisions have since been promoted into
`docs/apply_splice_spec.md`; these governance drafts now primarily control the
remaining implementation and release-closure work.

They agree on the following core points:

- implementation must begin only after stable contract promotion
- shared-substrate boundary lock is a formal gate, not an optional cleanup step
- parser/selection work and runtime work are distinct phases with separate review gates
- the full action basis is required for completion: eight transfer actions plus
  the `Delete Span` primitive
- connected-unit atomicity and partial success are release-level requirements
- diagnostics are part of the contract, not polish
- MCP/CLI parity is mandatory, not optional
- docs/examples drift is release-blocking
- “tests pass” means layered evidence, not only outer smoke tests

## Governance Chain

The current control chain now reads coherently:

1. Closure artifacts define the implementation-driving pre-code baseline.
2. `schedule_plan_draft.md` defines the phase order, gate order, and effort bands.
3. `acceptance_criteria_draft.md` defines what counts as passing each semantic
   and transport obligation.
4. `integration_review.md` and this note confirm that the drafts are mutually
   compatible enough to guide promotion into stable docs.

## What Is Strong

- The schedule draft avoids fake calendar precision and instead models effort,
  dependencies, risk checkpoints, and review gates.
- The acceptance draft is genuinely evidence-backed: each criterion has pass checks,
  evidence classes, negative expectations, and regression obligations.
- Both drafts correctly treat documentation truthfulness as part of the release
  contract.
- Both drafts keep `apply_splice` narrow and do not allow scope inflation to
  hide under abundant budget/time assumptions.

## Remaining Promotion Work

Before these governance drafts can stop living under `temporary/`, the project
 still needs to decide where their long-term stable homes belong.

Recommended promotion targets:

- phase/gate model:
  fold into an implementation plan or maintainer-facing execution plan
- acceptance and review criteria:
  fold into a stable acceptance document or a dedicated maintainer QA appendix
- source-of-truth precedence:
  fold into `maintainer_guide.md` or another stable governance doc

## Recommendation

The governance layer is now strong enough to stop asking “how should we manage
this work?” and start asking “which stable docs should absorb these controls
before coding begins?”

In short:

- closure baseline exists
- schedule baseline exists
- acceptance baseline exists
- the next management step is promotion, not more governance discovery
