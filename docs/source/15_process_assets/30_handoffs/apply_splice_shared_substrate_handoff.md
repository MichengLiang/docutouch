(process-assets-apply-splice-shared-substrate-handoff)=
# Apply Splice Shared Substrate Handoff

## Task Objective

在不引入 `apply_splice` public surface 的前提下，
先把 patch 栈里可复用的 lower correctness substrate 做第一步内部抽取，
为后续 shared mutation substrate 留出干净边界。

## Current Standing

- closed
- `mutation_support.rs` 已承载 path identity、normalization、affected-path merge，并对 splice runtime 复用
- deeper grouping / rollback 仍保持 runtime-owned，不再作为当前 handoff 的 deliverable

## Read These First

- {ref}`knowledge-architecture-apply-splice-architecture`
- {ref}`knowledge-operations-upstream-sync-and-compatibility`
- {ref}`process-assets-apply-splice-baseline-locking-work-package`
- {ref}`process-assets-apply-splice-implementation-stream-matrix`

## Allowed Edit Surface

- `codex-apply-patch/src/lib.rs`
- `codex-apply-patch/src/mutation_support.rs`
- `codex-apply-patch/tests/`

## Disallowed Areas

- any `apply_splice` parser/runtime/tool surface
- `docutouch-server/`
- `docutouch-core/src/splice_*`
- user-facing diagnostics wording changes

## Exact Deliverable

- a first internal extraction of splice-agnostic patch substrate helpers
- no observable patch behavior regression
- tests that still prove alias-aware grouping / affected-path behavior after extraction

## Verification Criteria

- `codex-apply-patch` tests pass
- no public `apply_patch` semantics change
- extracted module remains splice-agnostic rather than carrying patch-only vocabulary

## Escalation Conditions

- any extraction that would force a public behavior change
- any need to invent splice semantics inside the shared module
- any scope growth into transport or docs

## Report-Back Format

- files changed
- helpers extracted
- regression validation run
- intentionally deferred deeper substrate work
