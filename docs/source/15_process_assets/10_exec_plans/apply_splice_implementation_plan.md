(process-assets-apply-splice-implementation-plan)=
# `apply_splice` Implementation Plan

## Purpose

Maintain the live phase-by-phase implementation program for `apply_splice` after design
lock, without confusing this sequencing object for the stable product contract.

## Source Basis

- source material: `docs/archive2026年3月24日/temporary/apply_splice_implementation_plan.md`
- stable contract source: {ref}`knowledge-interfaces-apply-splice-spec`
- related closure/readiness inputs under `docs/archive2026年3月24日/temporary/apply_splice_closure/`

## Target Outcome

- preserve the completed design-lock phase as baseline context
- keep the completion path reviewable until parser, runtime, diagnostics, and prompt-facing
  surfaces all close truthfully
- sequence parser, runtime, diagnostics, and prompt-facing work without reopening tool
  identity decisions

## Scope

- implementation sequencing for grammar/parsing, selection resolution, atomic runtime,
  diagnostics/tests, and prompt-facing review
- review standards for the first implementation closure

## Non-Goals

- restating the stable product spec as accepted knowledge
- turning this page into a status record for already-finished work
- reopening narrow tool identity or action-basis decisions already locked elsewhere

## Milestones And Duration

| Milestone | Target Outcome | Current Standing | Expected Duration | Entry Condition | Exit Condition |
| --- | --- | --- | --- | --- | --- |
| M0 | Design lock preserved as completed baseline | completed | already complete | product-boundary decisions were closed | implementation may proceed without reopening tool identity |
| M1 | Grammar and parsing strategy is locked | completed | short implementation phase | design lock remains stable | canonical grammar and parser representation are ready for runtime work |
| M2 | Deterministic selection resolver exists | completed | short implementation phase | M1 complete | contiguous source/target selection resolution is implemented and validated |
| M3 | Safe atomic transfer runtime exists | completed | medium implementation phase | M2 complete | copy/move/delete, connected-unit grouping, same-file legality, and file-group safety are implemented |
| M4 | Diagnostics and tests close the main semantic risks | completed | medium implementation phase | M3 complete | drift, mismatch, overlap, target-state, write-stage, and partial-success cases are test-backed and reviewable |
| M5 | Prompt-facing guidance and final review package exist | completed | short finalization phase | M4 complete | user-facing contract, tool docs, parity checks, and acceptance summary are ready for outer-surface integration |

## Dependency Strategy

- keep implementation downstream of the stable spec and closure-owned readiness objects
- do not start runtime work until grammar/parsing and selection semantics are explicit
- use readiness hosts for release gates and QA obligations rather than overloading this
  plan with every acceptance detail

## Parallelization Plan

- parser representation work and low-risk review-package preparation may overlap only
  after the grammar is stable
- diagnostics and tests can grow alongside runtime work, but semantic edge cases should
  be locked before presentation text is finalized
- prompt-facing guidance should trail the implemented behavior instead of leading it

## Acceptance Strategy

- each remaining phase exits with a concrete deliverable rather than informal progress
  claims
- the review standard remains narrow, reference-preserving, and atomicity-aware
- phase transitions happen only when the prior semantic dependency is genuinely closed

## Risk And Replan Triggers

- if parser design reopens tool-boundary questions, stop and hand the issue back to a
  deliberation or spec host instead of continuing implementation sequencing
- if runtime work exposes unresolved same-file or newline-fidelity semantics, replan
  through readiness gates before broad coding continues
- if the remaining phases become large enough to need independent owners, split them into
  narrower work packages rather than burying all concurrency here

## Related Work Packages

- `apply_splice_baseline_locking_work_package`
- future parser/selection work packages
- future runtime/diagnostics work packages
- `apply_splice_implementation_schedule_plan`

## Related Records

- `30_records/60_status/apply_splice_implementation_status.md`
- future `30_records/50_audit/` review findings for semantic or QA failures
