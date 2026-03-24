# Line-Number Alignment Rollout Plan

## Purpose

Carry the active rollout for line-number and gutter alignment fixes as a maintained
execution plan for rendering polish that is deliberately limited to output presentation
and regression coverage.

## Source Basis

- source material: `docs/archive2026年3月24日/temporary/line_number_alignment_rollout_plan.md`
- affected runtime surfaces in `apply_patch` diagnostics and `search_text` grouped output

## Target Outcome

- `apply_patch` blame excerpt blocks keep a stable `|` column within each excerpt block
- `search_text` keeps a stable `|` column within each file-group block
- no semantic contract changes are introduced while presentation alignment becomes
  regression-tested

## Scope

- rendering changes for `apply_patch` diagnostics excerpt gutters
- rendering changes for `search_text` file-block line-number alignment
- related test and parity updates where exact output assertions exist

## Non-Goals

- global cross-file alignment
- changes to tool semantics, error codes, grouping, ranking, or omission accounting
- rewriting historical temporary output samples as active contract pages

## Milestones And Duration

| Milestone | Target Outcome | Expected Duration | Entry Condition | Exit Condition |
| --- | --- | --- | --- | --- |
| M1 | Reproduction anchors and affected assertions are identified | 0.5 day equivalent | misalignment cases are known conceptually | tests or internal repro anchors exist for both output families |
| M2 | `apply_patch` excerpt gutters align within each excerpt block | 0.5 day equivalent | M1 complete | 1-digit, 2-digit, and 3-digit excerpt cases keep a stable `|` column |
| M3 | `search_text` line numbers align within each file block | 0.5 day equivalent | M1 complete | mixed-width line numbers in preview/full file groups keep a stable `|` column |
| M4 | Full regression pass confirms no contract drift | 0.5 day equivalent | M2 and M3 complete | affected Rust tests pass and output semantics remain unchanged |

## Dependency Strategy

- isolate this work to presentation/rendering helpers and affected assertions
- treat semantic invariants as locked and use them as regression boundaries
- run full affected test suites before considering the rollout closed

## Parallelization Plan

- `apply_patch` excerpt alignment and `search_text` block alignment can be implemented in
  parallel because they touch different output families
- parity and assertion cleanup can run after the rendering changes land, but before the
  rollout is considered closed

## Acceptance Strategy

- the final output meets the four acceptance rules from the source plan: aligned
  excerpt blocks, aligned file-block groups, no semantic drift, and green affected tests
- output examples remain explanatory only; regression tests carry the durable proof

## Risk And Replan Triggers

- if exact-string tests prove too brittle, replan toward more stable structural
  assertions without weakening the visible contract
- if a rendering fix changes semantics or blame clarity, stop and treat it as a broader
  UX contract issue rather than a simple alignment rollout

## Related Work Packages

- narrow rendering/test packages for `patch_presentation`
- narrow rendering/test packages for `search_text`
- `ux_hardening_plan`

## Related Records

- future `30_records/60_status/` rollout closeout page
- future `30_records/50_audit/` regression review if output drift is found
