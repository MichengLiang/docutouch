(process-assets-apply-splice-implementation-schedule-plan)=
# `apply_splice` Implementation Schedule Plan

## Purpose

Provide the live phase/gate schedule from closure-complete, implementation-baseline-lock
state to `apply_splice` v1 completion, without confusing schedule management with the
stable tool contract.

## Source Basis

- source material: `docs/archive2026年3月24日/temporary/apply_splice_closure/schedule_plan_draft.md`
- stable contract source: {ref}`knowledge-interfaces-apply-splice-spec`
- closure/readiness inputs from `docs/archive2026年3月24日/temporary/apply_splice_closure/`

## Target Outcome

- one maintained critical-path schedule for implementation-baseline lock,
  architecture lock, substrate extraction, parser/runtime work, surface wiring, and
  final verification
- explicit review gates RG1 through RG7 instead of hidden sequencing assumptions
- a schedule that keeps implementation aligned with docs, tests, and release criteria

## Scope

- phase ordering for `apply_splice` v1 delivery
- review-gate entry/exit logic
- critical-path and overlap boundaries across the implementation effort

## Non-Goals

- reopening the closed product contract
- acting as the sole source of truth for grammar or semantics
- recording actual execution closeout after phases finish

## Milestones And Duration

| Milestone | Target Outcome | Current Standing | Expected Duration | Entry Condition | Exit Condition |
| --- | --- | --- | --- | --- | --- |
| M1 | Implementation baseline lock closes stable-authority gaps and phase ambiguity | completed | 1-3 idealized days | none | RG1 passes and engineering has one implementation-entry baseline |
| M2 | Shared-vs-owned architecture boundary is approved | completed | 1-3 idealized days | M1 complete | RG2 passes and module/API seams are reviewable |
| M3 | Shared mutation substrate is extracted without patch regression | completed | 4-6 idealized days | M2 complete | RG3 passes and shared helpers are reusable but still splice-agnostic |
| M4 | Parser and selection engine are implemented and test-backed | completed | 4-6 idealized days | M1 and M2 complete | RG4 passes and authored-surface/denotation behavior is deterministic |
| M5 | Runtime, diagnostics, and presentation cover all v1 actions | completed | 5-8 idealized days | M3 and M4 complete | RG5 passes and execution behavior matches the contract |
| M6 | MCP/CLI wiring and user docs are landed | completed | 2-3 idealized days | M5 complete | RG6 passes and supported surfaces match runtime behavior |
| M7 | Verification, hardening, and release closure are complete | completed | 3-5 idealized days | M6 complete | RG7 passes and the project can truthfully call the tool done |

## Dependency Strategy

- keep the contract and boundary phases linear so coding does not outrun normative
  clarity
- let parser/selection work begin only after grammar and boundary rules are stable
- treat docs and tool registration as release-critical downstream work, not optional
  cleanup after coding

## Parallelization Plan

- later coding overlap is allowed only where the source schedule already permits it:
  late Phase 4 after codec/API lock, and late Phase 6 on documentation drafting only
- no runtime work should start before RG3 and RG4 pass
- no release verification should start before surface wiring, tool docs, and parity are
  all stable enough for RG6

## Acceptance Strategy

- the schedule remains live only if phases and gates still answer the current path to v1
- each review gate carries concrete evidence rather than promise-only checkpoints
- cross-phase control rules remain explicit: no coding before RG1, no merge before the
  full action basis, parity, docs, and scenario matrix are closed

## Risk And Replan Triggers

- if any normative v1 rule still depends on temporary closure notes, stop at RG1 rather
  than letting implementation proceed speculatively
- if a proposed shared helper needs splice semantics to operate, stop at RG2 and shrink
  the shared boundary
- if patch regression appears during substrate extraction, repair the substrate before
  further splice runtime work continues
- if docs or examples still drift at the end, treat that as a release blocker rather
  than polish debt

## Related Work Packages

- `apply_splice_implementation_plan`
- future phase-specific work packages for substrate extraction, parser/runtime work, and
  docs/parity closure

## Related Records

- `30_records/60_status/apply_splice_implementation_status.md`
- future `30_records/50_audit/` review-gate findings if a gate fails or forces replan
