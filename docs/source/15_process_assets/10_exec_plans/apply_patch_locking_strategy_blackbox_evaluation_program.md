# `apply_patch` Locking Strategy Black-box Evaluation Program

## Purpose

Keep the long-window comparison between ordinary context matching and line-locked
authoring as an explicit live evaluation program rather than an unhosted temporary note.

## Source Basis

- source material: `docs/archive2026年3月24日/temporary/apply_patch_locking_strategy_blackbox_evaluation_program.md`
- current `apply_patch` diagnostics/runtime baseline

## Target Outcome

- one durable observation shape for paired strategy comparisons
- periodic comparative summaries that can be reviewed later
- an eventual default-strategy judgment grounded in observed repair-loop behavior rather
  than style preference

## Scope

- black-box comparison of strategy S1 and strategy S2 under real host/model behavior
- observation dimensions covering success rate, repair cost, host interaction cost,
  token cost, authoring quality, human review quality, and cross-model stability
- controlled task-family comparisons

## Non-Goals

- declaring a winner in advance
- changing the `apply_patch` grammar in this page
- repairing current defects directly inside this program
- mistaking aesthetic preference for evaluation evidence

## Milestones And Duration

| Milestone | Target Outcome | Expected Duration | Entry Condition | Exit Condition |
| --- | --- | --- | --- | --- |
| M1 | Freeze the shared logging shape and controlled variables for paired runs | short setup wave | both strategies are representable enough to observe | every observed task can be recorded with one structured record |
| M2 | Gather evidence across the required task families | medium observation window | M1 logging shape is stable | each task family has comparative observations under realistic use |
| M3 | Produce periodic summaries without closing the open questions too early | rolling during evaluation window | M2 observations exist | summaries expose repair-loop, ambiguity, and reviewability patterns without premature verdicts |
| M4 | Review default-strategy readiness based on joined evidence | after sufficient observation density | the open questions have enough observed evidence | the project can justify either keeping context matching first-path or promoting line-locked guidance with evidence |

## Dependency Strategy

- keep evaluation downstream of the currently shipped `apply_patch` behavior so the
  comparison stays black-box and operational
- compare paired runs only when workspace state, task objective, and baseline prompt
  policy are meaningfully controlled
- use future records for observed facts and keep this page focused on the evaluation plan

## Parallelization Plan

- different task families can be observed in parallel once the logging shape is stable
- summaries can be written independently from future observation batches
- cross-model observation can proceed in parallel with human-review-quality analysis as
  long as record fields stay stable

## Acceptance Strategy

- the program remains useful only if it preserves durable evidence rather than ad hoc
  anecdotes
- every required task family has a reviewable observation path
- the eventual default-strategy decision can cite this program's evidence model directly

## Risk And Replan Triggers

- if the strategy definitions themselves change materially, earlier observations may need
  to be partitioned rather than merged blindly
- if task families or logging fields prove too vague to compare honestly, the program
  should tighten its observation shape before collecting more data
- if one strategy becomes obviously unusable under real repair loops, the program may
  shift from open comparison to deprecation-readiness review

## Related Work Packages

- future observation-log maintenance packages
- future comparative-summary writing packages
- future default-strategy decision review package

## Related Records

- future `30_records/50_audit/` comparative findings
- future `30_records/60_status/` strategy-decision closeout pages
