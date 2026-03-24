(process-assets-apply-splice-presentation-transport-handoff)=
# Apply Splice Presentation And Transport Handoff

## Task Objective

在 runtime core 稳定后，
把 `apply_splice` 的 user-visible presentation、tool docs 与 server/CLI transport wiring 接上，
并维持与 runtime 行为一致的 outer-surface contract。

## Current Standing

- closed
- presentation layer、transport wiring、tool docs 与 CLI/MCP parity 已全部落地
- representative failure output 现在保留 splice file/source blame、committed `A/M/D` 与 failed-unit accounting

## Read These First

- {ref}`knowledge-interfaces-apply-splice-spec`
- {ref}`knowledge-architecture-apply-splice-architecture`
- {ref}`process-assets-apply-splice-runtime-handoff`
- {ref}`process-assets-apply-splice-implementation-stream-matrix`
- {ref}`process-assets-readiness-index`

## Allowed Edit Surface

- `docutouch-core/src/lib.rs`
- `docutouch-core/src/splice_presentation.rs`
- `docutouch-server/src/cli.rs`
- `docutouch-server/src/tool_service.rs`
- `docutouch-server/tests/cli_smoke.rs`
- `docutouch-server/tests/stdio_smoke.rs`
- `docutouch-server/tool_docs/apply_splice.md`

## Disallowed Areas

- patch tool docs or patch transport semantics
- reopening runtime semantics already closed by tests
- widening tool boundary beyond current action basis

## Exact Deliverable

- `apply_splice` presentation layer aligned with DocuTouch family style
- MCP/CLI surface wiring for `apply_splice`
- tool docs that match implemented behavior
- parity tests for representative success/failure cases

## Verification Criteria

- runtime behavior and outer-surface text agree
- CLI/MCP parity tests pass
- tool docs examples map to tested behavior
- no regression to existing patch transport behavior

## Escalation Conditions

- runtime semantics still moving underneath the presentation work
- need to reopen stable boundary or action basis
- any required patch behavior change outside parity-safe refactoring

## Report-Back Format

- files changed
- presentation/transport surfaces added
- tests run
- deferred non-v1 follow-up items
