# DocuTouch UX Hardening Plan

## Purpose

Turn the remaining UX cleanup into a maintained execution object instead of leaving the
program split between shipped wave notes, temporary decisions, and ambient memory.

## Source Basis

- source material: `docs/archive2026年3月24日/ux_hardening_plan.md`
- source material: `docs/archive2026年3月24日/roadmap.md`
- runtime-facing diagnostics and tool-contract materials referenced from the source plan

## Target Outcome

- preserve the already-shipped wave history only as status context
- keep the remaining UX work explicit: residual diagnostics cleanup, failed-patch-source
  wording/test sync, and post-ship search experience optimization
- maintain one canonical process host for the ordering of UX hardening work

## Scope

- workspace/path UX consistency
- diagnostics and repair-surface clarity
- success/no-op parity and wording stability
- doc/runtime synchronization for high-frequency tool flows
- search experience optimization after the earlier correctness waves

## Non-Goals

- redesigning the entire product surface
- adding broad new tool categories
- rewriting already-shipped history into fake open work
- treating historical artifact-retirement context as a new active spec

## Milestones And Duration

| Milestone | Target Outcome | Expected Duration | Entry Condition | Exit Condition |
| --- | --- | --- | --- | --- |
| M1 | Preserve wave history while isolating only the remaining open UX debt | same-round host migration plus rolling maintenance | source plan contains mixed implemented and unresolved material | shipped waves are clearly status context and remaining work is explicit |
| M2 | Close residual post-wave diagnostics/doc sync gaps | short follow-on wave | parity and diagnostics baselines already exist | failed-patch-source persistence, per-group pointers, multiline cause indentation, and related tests/docs are synchronized |
| M3 | Keep `search_text` optimization as the next active UX gain after patch/workspace stabilization | medium rolling wave | earlier correctness and parity work are not regressing | grouped, lower-noise search experience has an owned plan and no longer sits as an ambient future note |

## Dependency Strategy

- treat Waves 0 through 4 as implemented baseline, not as open implementation work
- let remaining diagnostics cleanup follow the accepted runtime contract instead of
  reopening finished product decisions
- treat search optimization as downstream of the stabilized patch and workspace surface

## Parallelization Plan

- residual diagnostics/doc sync can run in parallel with targeted test cleanup when they
  share one contract owner
- `search_text` optimization work can be prepared in parallel, but should not preempt
  higher-risk repair-surface synchronization if both touch shared docs or examples
- historical-doc cleanup can run beside runtime work only when it remains purely
  superseded-history marking rather than contract editing

## Acceptance Strategy

- the maintained plan distinguishes implemented waves from still-open work without
  pretending the whole program is historically closed
- remaining UX work maps cleanly to explicit tasks and review checkpoints
- docs continue to describe real runtime behavior rather than a superseded or
  aspirational surface

## Risk And Replan Triggers

- if diagnostics improvements require contract changes rather than implementation sync,
  this page must hand work back to a spec-bearing or deliberation host instead of silently
  expanding scope
- if `search_text` optimization becomes blocked on deeper architecture work, the active
  portion of this plan should split into a separate execution or work-package host
- if more waves move to completed-only status with no open remainder, this plan should be
  evaluated for disposition into `30_records/`

## Related Work Packages

- `line_number_alignment_rollout_plan`
- future `search_text` optimization packages under `20_work_packages/`
- future diagnostics/doc-sync packages derived from the remaining unresolved items

## Related Records

- future `30_records/60_status/` wave closeout pages
- future `30_records/50_audit/` doc/runtime drift reviews
