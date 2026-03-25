(process-assets-apply-patch-line-number-assist-baseline-locking-work-package)=
# Apply Patch Line-Number-Assist Baseline Locking Work Package

## Objective

在 broad implementation 开始前，
先把 `apply_patch` line-number-assisted locking 的 authority baseline、candidate semantics、readiness gate 与 execution streams 锁到可并发执行的程度，
避免后续 coding 再回头裁决“public form 教什么、parser support 支持什么、runtime 应按什么快照解释”。

## Upstream Plan

- {ref}`process-assets-apply-patch-line-number-assist-rollout-plan`

## Required Inputs

- {ref}`knowledge-decisions-apply-patch-numbered-anchor-guidance-rationale`
- {ref}`deliberation-candidate-specs-apply-patch-line-number-assisted-locking-draft`
- {ref}`knowledge-interfaces-apply-patch-semantics`
- {ref}`records-audit-apply-patch-anchor-semantics-investigation`
- {ref}`knowledge-operations-testing-and-tool-admission`
- `docutouch-server/tool_docs/apply_patch.md`
- `codex-apply-patch/apply_patch_tool_instructions.md`

## Deliverables

- primary deliverable: stable candidate-spec host for line-number-assisted locking under `20_deliberation/40_candidate_specs/`
- primary deliverable: accepted decision host that fixes the public canonical form under `10_knowledge/60_decisions/`
- primary deliverable: rollout plan, acceptance criteria, and execution matrix under `15_process_assets/`
- supporting update: implementation-facing stream inventory that can be handed to multiple agents without reopening product-boundary or prompt-surface questions
- expected record sink: future `30_records/60_status/` page once rollout closes

## Current Standing

- closed
- baseline authority 已完成锁定，并已实际支撑后续 parser/runtime/presentation/parity coding wave

## Dependencies

- the current single-tool identity of `apply_patch` must remain closed while this package runs
- code investigation must complete before stream ownership is finalized
- prompt-facing canonical form must lock before tool-doc rewrites and parser/runtime work are split across owners

## Task Breakdown

1. write the accepted rationale that fixes `@@ N | visible text` as the canonical public auxiliary-location form
2. write the candidate spec that separates parser support set, prompt-preferred subset, and host escalation guidance
3. reconcile the main docs so engineers can tell which surfaces are accepted, candidate, and rollout-only without archive-first reading
4. inspect parser/runtime/presentation/test surfaces and convert them into explicit execution streams
5. land the rollout plan, acceptance criteria, and execution matrix that will authorize parallel implementation

## Parallelization Notes

- task 1 and task 2 may overlap, but both must finish before task 3 closes
- task 4 depends on tasks 1 through 3 producing a stable contract baseline
- task 5 depends on task 4, because stream planning without real code-surface inspection would be speculative

## Owner Type

- mixed: principal orchestration by main agent, with parallel implementation support from sub-agents after baseline lock closes

## Acceptance

- a reviewer can answer “what is canonical public guidance, what wider parser support is allowed, and what runtime semantics now govern numbering” without reopening archive-only notes
- no new page reopens stacked-`@@` hierarchy or second-tool identity paths
- the execution streams are explicit enough to split parser/source-map, runtime, presentation/tool-doc, and QA/parity work across multiple agents
- readiness language and execution language now point at the same candidate contract surface

## Exit Route

- feed concrete implementation streams into matrix and agent briefs
- allow subsequent coding work to treat the new decision/spec bundle as the implementation-entry authority
- record rollout closure in `30_records/60_status/apply_patch_line_number_assist_rollout_status.md`
