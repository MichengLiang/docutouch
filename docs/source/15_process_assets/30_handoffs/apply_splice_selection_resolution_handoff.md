(process-assets-apply-splice-selection-resolution-handoff)=
# Apply Splice Selection Resolution Handoff

## Task Objective

在已有 parser/selection grammar 基础上，
实现 `apply_splice` selection block 对真实文本内容的 deterministic resolution，
把 numbered excerpt + omission token 转成可验证的 contiguous interval。

## Current Standing

- closed
- deterministic resolution 已落地到 `docutouch-core/src/splice_selection.rs`
- resolution tests 已落地到 `docutouch-core/tests/splice_selection_resolution.rs`

## Read These First

- {ref}`knowledge-interfaces-apply-splice-spec`
- {ref}`knowledge-architecture-apply-splice-architecture`
- {ref}`deliberation-candidate-specs-apply-splice-formal-semantics-draft`
- {ref}`process-assets-apply-splice-parser-selection-handoff`
- {ref}`process-assets-apply-splice-implementation-stream-matrix`

## Allowed Edit Surface

- `docutouch-core/src/lib.rs`
- `docutouch-core/src/splice_selection.rs`
- `docutouch-core/tests/splice_selection_resolution.rs`

## Disallowed Areas

- runtime filesystem mutation logic
- `docutouch-server/`
- `codex-apply-patch/`
- diagnostics family final naming
- transport/tool docs

## Exact Deliverable

- selection resolver that validates numbered excerpt blocks against concrete file text
- deterministic contiguous interval result for legal selections
- explicit mismatch failure paths for source/target visible-content disagreement or impossible denotation
- tests covering exact-match resolution, omission-backed gaps, mismatch, wrong line-number anchor, and forbidden non-contiguous denotation

## Verification Criteria

- targeted selection-resolution tests pass
- existing parser/selection tests continue to pass
- no write-path or transport behavior is introduced
- the implementation stays narrow: resolution only, not transfer semantics

## Escalation Conditions

- ambiguity about whether a case is selection resolution or runtime legality
- any need to decide final overlap policy, byte-fidelity policy, or partial-success behavior
- any need to define final user-facing diagnostics vocabulary beyond local resolver errors

## Report-Back Format

- files changed
- interval semantics implemented
- tests added and run
- deferred runtime-only items
