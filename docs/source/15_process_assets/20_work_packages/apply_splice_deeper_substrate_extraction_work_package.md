(process-assets-apply-splice-deeper-substrate-extraction-work-package)=
# Apply Splice Deeper Substrate Extraction Work Package

## Objective

在已完成的 engineering hardening wave 之后，
继续推进剩余的 deeper substrate seams，
优先收拢 patch/splice 之间仍然重复拥有的 generic path/grouping/diff/commit mechanics，
并保持 `apply_splice` 的语义层边界不被错误下沉。

## Upstream Plan

- {ref}`process-assets-apply-splice-engineering-hardening-plan`
- {ref}`process-assets-apply-splice-engineering-hardening-readiness`

## Required Inputs

- {ref}`knowledge-architecture-apply-splice-architecture`
- {ref}`knowledge-operations-upstream-sync-and-compatibility`
- {ref}`records-status-apply-splice-engineering-hardening`
- current `codex-apply-patch` / `docutouch-core` runtime implementations

## Deliverables

- one narrower shared substrate for any remaining generic path/grouping/diff/commit helpers that
  can genuinely serve both patch and splice
- corresponding integration of that substrate into patch and splice runtimes
- regression tests that prove the extracted helper still preserves current contracts
- explicit note of any residual seam that still should not be shared yet

## Current Standing

- closed
- first-wave hardening already closed doctrine propagation, selection-authority cleanup,
  presentation hardening, transport-shell hardening, and shared runtime path resolution
- affected-path diff、connected-unit grouping 与 generic filesystem transaction mechanics 已收入共享 substrate
- splice-specific failed-unit `committed` reporting 仍刻意保留在 splice-owned layer，而没有被错误下沉为 generic helper

## Dependencies

- product-boundary doctrine must remain fixed while this package runs
- any extracted helper must remain splice-agnostic and patch-agnostic at the semantic layer
- QA must re-run the workspace tests after each deeper extraction slice

## Task Breakdown

1. identify which remaining seam is truly substrate-level rather than semantic disguise
2. extract the narrowest reusable helper for that seam
3. integrate patch and splice callers onto the helper without changing public wording
4. add or tighten regression coverage for the moved contract
5. record any still-unshared seam explicitly instead of silently treating it as done

## Parallelization Notes

- deeper substrate extraction should stay single-owner when multiple candidate seams touch the same
  core runtime files
- docs/status updates should trail the code extraction and verification, not lead it

## Owner Type

- main agent plus one focused implementation worker for the shared substrate slice

## Acceptance

- at least one remaining deeper substrate seam is materially closed rather than merely renamed
- patch/splice public contracts remain green under workspace `cargo test`
- no splice-owned diagnostics or same-file policy leak into the generic layer
- remaining seams, if any, are explicitly recorded instead of hidden

## Exit Route

- update `30_records/60_status/apply_splice_engineering_hardening_status.md`
- mark this package closed and leave only the intentionally unshared splice-owned residue in status notes
