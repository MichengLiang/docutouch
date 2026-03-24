(process-assets-apply-splice-parser-selection-handoff)=
# Apply Splice Parser And Selection Handoff

## Task Objective

实现 `apply_splice` 的第一批语义基础设施，
仅覆盖 envelope/header parsing 与 numbered excerpt selection parsing/validation，
不越界到 runtime 写入或 transport wiring。

## Current Standing

- closed
- parser / authored grammar 已落地到 `docutouch-core/src/splice_program.rs`
- 首批 parser/selection tests 已落地到 `docutouch-core/tests/splice_parser_selection.rs`

## Read These First

- {ref}`knowledge-interfaces-apply-splice-spec`
- {ref}`knowledge-architecture-apply-splice-architecture`
- {ref}`deliberation-candidate-specs-apply-splice-formal-semantics-draft`
- {ref}`process-assets-apply-splice-baseline-locking-work-package`
- {ref}`process-assets-apply-splice-implementation-stream-matrix`

## Allowed Edit Surface

- `docutouch-core/src/lib.rs`
- `docutouch-core/src/splice_program.rs`
- `docutouch-core/src/splice_selection.rs`
- `docutouch-core/tests/splice_parser_selection.rs`

## Disallowed Areas

- `docutouch-core/src/patch_*`
- `docutouch-server/`
- `codex-apply-patch/`
- runtime filesystem mutation logic
- transport registration, tool docs, outer-surface UX wording

## Exact Deliverable

- compilable parser/AST skeleton for canonical `apply_splice` envelope and action headers
- compilable selection parser for numbered excerpt blocks with source/target omission token support
- explicit rejection path for horizontal truncation in selections
- parser/selection tests covering positive and negative baseline cases

## Verification Criteria

- targeted tests for parser/selection pass
- no runtime write behavior is introduced
- no transport surface is changed
- the implementation stays within the currently stable or narrowly accepted candidate semantics

## Escalation Conditions

- exact grammar ambiguity that cannot be resolved conservatively from current stable/candidate docs
- any need to touch runtime write logic, patch-owned internals, or server wiring
- any requirement to define final diagnostics wording beyond parser/selection-local failures

## Report-Back Format

- files changed
- implemented rules
- tests added and run
- intentionally deferred items and why they were deferred
