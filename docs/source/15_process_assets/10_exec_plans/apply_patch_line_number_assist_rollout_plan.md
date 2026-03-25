(process-assets-apply-patch-line-number-assist-rollout-plan)=
# Apply Patch Line-Number-Assist Rollout Plan

## Purpose

将 `apply_patch` line-number-assisted locking 从 candidate spec 推进到 landed implementation、tool-doc sync 与 transport-visible contract closure。

## Source Basis

- {ref}`knowledge-decisions-apply-patch-numbered-anchor-guidance-rationale`
- {ref}`deliberation-candidate-specs-apply-patch-line-number-assisted-locking-draft`
- {ref}`knowledge-interfaces-apply-patch-semantics`
- {ref}`knowledge-operations-testing-and-tool-admission`
- {ref}`records-audit-apply-patch-anchor-semantics-investigation`

## Target Outcome

- parser/runtime 支持 line-number-assisted old-side evidence；
- prompt-facing public form 收敛到 `@@ N | visible text`；
- tool docs、stable docs、tests 与 examples 同步到真实行为；
- implementation stream 可以并发推进而不重新打开 `apply_patch` object identity。

## Scope

- parser grammar and source-map changes for numbered old-side evidence
- runtime matching semantics for numbered old-side locking
- diagnostics, presentation, and repair guidance updates
- MCP/CLI/tool-doc/example parity
- rollout readiness and permanent regression coverage

## Non-Goals

- 发明第二个 public patch tool
- 把 `apply_patch` 重写成 `apply_splice`-style selection language
- 把 dense numbered evidence 提升成默认 public teaching surface
- 在本页内重写 final accepted interface semantics

## Milestones And Duration

| Milestone | Target Outcome | Current Standing | Expected Duration | Entry Condition | Exit Condition |
| --- | --- | --- | --- | --- | --- |
| M0 | baseline authority is explicit | completed | short setup wave | candidate spec and decision hosts exist | implementation can cite one stable baseline without archive-first reading |
| M1 | parser/source-map support exists | completed | short implementation wave | M0 complete | numbered anchor and numbered old-side evidence parse truthfully and preserve authored blame locations |
| M2 | runtime semantics are closed | completed | medium implementation wave | M1 complete | numbered old-side evidence is interpreted against original snapshot and mismatch behavior is deterministic |
| M3 | diagnostics, presentation, and tool docs close truthfully | completed | medium implementation wave | M2 complete | visible failures, canonical examples, and injected docs match implemented behavior |
| M4 | transport parity and permanent regression coverage close | completed | medium verification wave | M3 complete | CLI/MCP parity, representative examples, and durable tests are green |
| M5 | stable interface wording can be promoted or refreshed | completed | short finalization wave | M4 complete | accepted docs can state the landed contract without speculative language |

## Dependency Strategy

- baseline authority must close before code is split across owners
- parser/source-map work precedes runtime matching because runtime semantics depend on authored evidence shape
- presentation/tool-doc wording trails implemented behavior rather than leading it
- stable interface semantics update waits until transport-visible behavior is test-backed

## Parallelization Plan

- parser/source-map and runtime prototyping may overlap only after M0 locks the candidate contract surface
- diagnostics/presentation and tool-doc sync can proceed in parallel once runtime mismatch categories stabilize
- transport parity and examples can run in parallel with final regression expansion if both target the same landed contract

## Acceptance Strategy

- every milestone exits with a reviewable deliverable rather than informal confidence
- public recommendation stays narrow even if parser support is wider
- no phase reopens stacked-`@@` hierarchy or second-tool identity paths
- all visible examples map to tested behavior

## Risk And Replan Triggers

- if parser support and prompt guidance start drifting again, stop and repair authority before more coding
- if original-snapshot interpretation proves incompatible with current multi-chunk runtime assumptions, replan through a narrower work package instead of improvising in-place
- if dense numbered-evidence support creates disproportionate complexity, keep the public canonical form and explicitly bound the wider parser surface

## Related Work Packages

- `apply_patch_line_number_assist_baseline_locking_work_package`
- future parser/runtime stream packages
- future diagnostics/parity stream packages

## Related Readiness Hosts

- {ref}`process-assets-apply-patch-line-number-assist-acceptance-criteria`

## Related Records

- future `30_records/50_audit/` findings for follow-on review
- `30_records/60_status/apply_patch_line_number_assist_rollout_status.md`
