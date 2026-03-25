(process-assets-apply-splice-runtime-handoff)=
# Apply Splice Runtime Handoff

## Task Objective

实现 `apply_splice` 的 runtime core，
先把 action semantics、same-file rule、target existence 与 byte-preserving transfer 做成可测试的执行层，
不越界到 transport wiring 或最终 user-facing diagnostics/presentation。

## Current Standing

- closed
- runtime core、same-file rule、target existence、byte-preserving transfer、connected-unit grouping 与 partial-success accounting 已闭合
- splice diagnostics family 与 authored-line blame 已直接落到 runtime-owned surface

## Read These First

- {ref}`knowledge-interfaces-apply-splice-spec`
- {ref}`knowledge-architecture-apply-splice-architecture`
- {ref}`deliberation-candidate-specs-apply-splice-formal-semantics-draft`
- {ref}`process-assets-apply-splice-selection-resolution-handoff`
- {ref}`process-assets-apply-splice-shared-substrate-handoff`
- {ref}`process-assets-apply-splice-implementation-stream-matrix`

## Allowed Edit Surface

- `docutouch-core/src/lib.rs`
- `docutouch-core/src/splice_runtime.rs`
- `docutouch-core/src/splice_selection.rs`
- `docutouch-core/tests/splice_runtime.rs`

## Disallowed Areas

- `docutouch-server/`
- `docutouch-core/src/patch_*`
- final MCP/CLI tool registration
- widening tool boundary beyond the accepted splice action basis

## Exact Deliverable

- runtime core that can execute parsed splice actions against concrete file state
- support for the current action basis at the semantic layer
- same-file original-snapshot interpretation and overlap rejection only when the same-file source range overlaps the anchored target range
- append-create behavior for missing destination files and failure for missing anchored targets
- tests that assert resulting filesystem state and raw transferred content behavior

## Verification Criteria

- runtime tests pass
- existing parser/selection tests continue to pass
- moved/copied spans preserve source bytes rather than reconstructing text loosely
- no transport or user-facing presentation coupling is introduced

## Escalation Conditions

- any need to finalize `SPLICE_*` code family or transport-visible wording
- any uncertainty about connected-unit grouping beyond the current narrow runtime batch
- any scope growth into `docutouch-server` or tool docs

## Report-Back Format

- files changed
- actions/semantics implemented
- tests added and run
- deferred non-v1 follow-up items
