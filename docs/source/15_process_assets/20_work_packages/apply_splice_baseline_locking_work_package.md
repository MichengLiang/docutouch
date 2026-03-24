(process-assets-apply-splice-baseline-locking-work-package)=
# Apply Splice Baseline Locking Work Package

## Objective

在 broad implementation 开始前，
先把 `apply_splice` 的 implementation-entry baseline 收紧到可并发执行的程度，
避免团队在 stable contract、candidate semantics、readiness gate 与 archive source material 之间来回裁决。

## Upstream Plan

- {ref}`process-assets-apply-splice-implementation-plan`
- {ref}`process-assets-apply-splice-implementation-schedule-plan`

## Required Inputs

- {ref}`knowledge-interfaces-apply-splice-spec`
- {ref}`knowledge-decisions-apply-splice-apply-patch-separation-rationale`
- {ref}`deliberation-proposals-apply-splice-technical-investigation`
- {ref}`deliberation-candidate-specs-apply-splice-formal-semantics-draft`
- {ref}`knowledge-operations-upstream-sync-and-compatibility`
- {ref}`knowledge-operations-testing-and-tool-admission`
- {ref}`records-status-apply-splice-closure`
- {ref}`records-audit-apply-splice-integration-review`

## Deliverables

- primary deliverable: stable `apply_splice` architecture host under `10_knowledge/50_architecture/`
- primary deliverable: accepted operations wording that treats vendored `codex-apply-patch` as internal substrate rather than downstream architecture leash
- primary deliverable: refreshed implementation baseline that distinguishes stable authority from candidate/readiness material
- supporting update: schedule / status / gate language reconciled so engineering execution does not inherit stale phase assumptions
- supporting update: implementation-facing task inventory that can be handed to multiple agents without reopening product boundary questions
- expected record sink: future `30_records/50_audit/` or `30_records/60_status/` pages if baseline lock reveals unresolved blockers

## Current Standing

- closed
- baseline lock deliverables are landed and have already handed off parser / selection / shared substrate / runtime / presentation / transport work
- downstream implementation and release verification are now recorded in `30_records/60_status/apply_splice_implementation_status.md`

## Dependencies

- stable product boundary must remain closed while this package runs
- architecture boundary work must finish before large runtime coding begins
- testing expectations must stay aligned with the existing three-layer discipline and recursive QA model

## Task Breakdown

1. sync governance wording so the repository explicitly treats vendored `codex-apply-patch` as an internal substrate rather than a downstream architecture leash
2. land a stable `apply_splice` architecture host that names the shared-vs-owned boundary
3. reconcile implementation-entry authority so engineers can tell which semantics are already stable and which still remain candidate or readiness material
4. refresh schedule / gate language so phase names match the actual closure state and no longer imply already-finished promotion work is still pending
5. produce the first implementation-facing stream inventory for parser / selection / runtime / presentation / transport / QA work

## Parallelization Notes

- task 1 and task 2 may overlap if both keep the closed product boundary intact
- task 3 depends on task 1 and task 2 producing a stable governance and architecture baseline
- task 4 depends on task 3, because schedule repair should follow authority repair rather than guess at it
- task 5 can begin once tasks 2 through 4 have removed baseline ambiguity

## Owner Type

- mixed: principal orchestration by main agent, with parallel analysis / implementation support from sub-agents

## Acceptance

- a reviewer can answer “what is already stable, what is still candidate, and what now authorizes implementation” without consulting archive-only material first
- `apply_splice` shared-vs-owned boundary is written in an accepted architecture host
- the project docs explicitly state that upstream disclosure remains required, but upstream closeness does not control downstream `apply_splice` architecture
- the baseline task inventory is explicit enough to split into parser / selection / runtime / presentation / transport / QA streams
- no new document reopens the closed `apply_patch` / `apply_splice` product-boundary decision

## Exit Route

- hand off concrete implementation slices to follow-up work packages or agent briefs
- feed updated readiness / schedule hosts with the locked baseline
- allow subsequent coding work to treat the new architecture and operations wording as the governing authority
- record subsequent execution closure in `30_records/60_status/apply_splice_implementation_status.md`
