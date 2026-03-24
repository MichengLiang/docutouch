# DocuTouch Roadmap

## Purpose

Keep the project's live priority order explicit so product evolution does not drift
toward feature sprawl, dated commitments, or ambient preferences.

## Source Basis

- source material: `docs/archive2026年3月24日/roadmap.md`
- source material: `docs/archive2026年3月24日/ux_hardening_plan.md`
- `docs/source/00_meta/120_process_assets_and_authority_conversion_policy.md`
- `docs/source/00_meta/130_build_root_and_authority_role_distinction.md`

## Target Outcome

- near-term work stays anchored on correctness, UX/runtime/doc sync, and main-path
  simplicity
- medium-term work is visible without being confused for immediate commitment
- exploratory topics remain explicitly non-priority until the trigger conditions exist
- roadmap updates remain rule-driven rather than chat-driven

## Scope

- product evolution priorities
- sequencing of active and deferred investments
- explicit non-goals and reprioritization triggers

## Non-Goals

- date commitments
- sprint-by-sprint status tracking
- actual implementation closeout
- generic idea backlog collection

## Milestones And Duration

| Milestone | Target Outcome | Expected Duration | Entry Condition | Exit Condition |
| --- | --- | --- | --- | --- |
| M1 | Keep the current near-term track visible: `apply_patch` correctness, doc/runtime sync, and low-noise main-path work | rolling current-phase host | shipped baseline is stable enough to prioritize polish over tool-count growth | roadmap still reflects the highest-value live priorities |
| M2 | Keep medium-term investments explicit without promoting them to immediate commitments | rolling follow-on planning | M1 priorities are already visible and defended | warning/diagnostics, audit-boundary cleanup, core file-tool noise reduction, and `search_text` optimization remain reviewable as next-layer work |
| M3 | Keep exploratory topics fenced as conditional, not ambient pressure | rolling background horizon | future-facing topics exist but lack forcing conditions | CLI priority, strict-profile work, and new-tool expansion remain conditional on explicit trigger evidence |

## Dependency Strategy

- use the shipped runtime and accepted contract surface as the baseline for priority
  judgment
- let focused execution plans carry individual delivery details while this roadmap keeps
  only ordering and policy-level rationale
- revise the roadmap when product stance changes, not when one implementation task
  merely finishes

## Parallelization Plan

- correctness hardening and doc/runtime sync can proceed in parallel as long as they do
  not change the same contract text blindly
- low-noise file-tool optimization can run beside diagnostics work when shared behavior
  remains explicit
- exploratory CLI or strict-profile work stays parked until the documented trigger
  conditions are met

## Acceptance Strategy

- the roadmap continues to answer what is highest priority now, what is deferred, and
  what is explicitly out of scope
- live execution plans can map back to one roadmap bucket without contradiction
- new feature proposals are reviewable against this page before priority expansion is
  accepted

## Risk And Replan Triggers

- if correctness work uncovers a deeper substrate issue, execution may need to move from
  polish to architecture hardening
- if CLI usage becomes a dominant real workflow, the exploratory CLI section should be
  promoted out of the long-horizon bucket
- if search or read orchestration becomes the main token-cost bottleneck, `search_text`
  optimization may need to move ahead of lower-value polish work
- if the project begins carrying unrecorded upstream divergence, the roadmap must be
  tightened before more surface expansion happens

## Related Work Packages

- `ux_hardening_plan`
- `engineering_quality_wave_20260323_plan`
- `line_number_alignment_rollout_plan`
- `apply_patch_locking_strategy_blackbox_evaluation_program`

## Related Records

- future `30_records/60_status/` closeout pages for completed roadmap items
- future `30_records/50_audit/` review pages for priority or contract drift
