# Engineering Quality Wave Plan (2026-03-23)

## Purpose

Host the live next-wave engineering quality program that follows the earlier
diagnostics-contract sync work, focusing on ownership cleanup and maintainability rather
than new user-facing behavior.

## Source Basis

- source material: `docs/archive2026年3月24日/temporary/engineering_quality_wave_20260323/plan.md`
- prior diagnostics contract sync wave outputs referenced by the source plan
- `docutouch` transport/runtime architecture as described in repository docs and code

## Target Outcome

- CLI and MCP share a clearer semantic adapter boundary
- patch-source provenance has one primary owner
- smoke-test child lifecycle is guarded by reusable helpers
- execution-anchor and display-anchor responsibilities are less entangled
- maintainer-facing docs match the post-wave codebase

## Scope

- shared patch invocation adapter cleanup
- provenance ownership cleanup
- smoke-test harness hardening
- anchor-context decoupling
- post-refactor contract re-verification
- maintainer-facing documentation closure for this wave

## Non-Goals

- redesigning the accepted diagnostics contract again
- reopening committed partial-failure behavior
- removing failed patch-source persistence
- adding new end-user-facing tools
- style-only refactors without maintenance payoff

## Milestones And Duration

| Milestone | Target Outcome | Expected Duration | Entry Condition | Exit Condition |
| --- | --- | --- | --- | --- |
| M1 | Shared patch invocation adapter direction is landed | short wave slice | diagnostics contract baseline is already accepted | CLI/MCP semantic triggering flows through one clearer adapter boundary |
| M2 | Patch-source provenance has one obvious owner | short wave slice | M1 or equivalent boundary clarity exists | duplicated precedence logic is reduced and tests cover the intended cases |
| M3 | Smoke-test lifecycle helpers are hardened | short wave slice | current smoke paths are identified | important spawned-process paths have timeout and cleanup policy |
| M4 | Anchor-context ownership is clarified | short wave slice | M1 and M2 reveal the relevant seams | execution-anchor versus display-anchor responsibilities are easier to explain |
| M5 | Contract re-verification and maintainer-doc closure complete the wave | end-of-wave integration pass | M1-M4 converge | tests, example run, and maintainer-facing docs all align with the changed architecture |

## Dependency Strategy

- treat the accepted diagnostics contract as locked input, not as active discovery
- start ownership workstreams before docs cleanup so documentation follows settled seams
- keep contract re-verification and documentation closure as explicit end-of-wave gates

## Parallelization Plan

- the source plan's recommended four-worker split remains valid: shared adapter,
  provenance ownership, harness hardening, and read-first review/re-verification
- anchor cleanup may begin only after the earlier ownership direction becomes clear
- docs-only follow-up work should wait until the code ownership changes settle

## Acceptance Strategy

- the wave is complete only when semantic duplication is reduced rather than merely moved
- future drift risk is materially lower, not merely hidden behind current green tests
- the accepted user-facing diagnostics contract remains intact after refactoring
- the source plan's ship criteria remain satisfied: core/server tests pass, example run
  still demonstrates the intended diagnostics surface, and maintainer docs stop
  overstating the shipped transport/authority state

## Risk And Replan Triggers

- if adapter cleanup or provenance cleanup reopens user-facing contract behavior, stop
  and reclassify the work instead of smuggling contract change into a quality wave
- if extraction or decoupling introduces patch regressions, repair those regressions
  before continuing to later wave slices
- if documentation closure reveals unresolved architecture disagreement, do not treat doc
  editing as sufficient; reopen the relevant implementation slice first

## Related Work Packages

- future worker-owned packages for adapter, provenance, harness, and anchor cleanup
- future documentation alignment package derived from Workstream F

## Related Records

- future `30_records/60_status/` wave closeout page
- future `30_records/50_audit/` review findings from the re-verification pass
