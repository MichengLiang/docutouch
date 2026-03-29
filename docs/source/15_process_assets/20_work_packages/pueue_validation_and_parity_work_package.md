(process-assets-work-package-pueue-validation-and-parity)=
# Pueue Validation And Parity Work Package

## Objective

在 runtime substrate、wait surface 与 log-handle surface 都落地后，
统一完成 parity verification、docs build verification、kickoff readiness closeout 与 records sink 指向。

## Upstream Plan

- {ref}`process-assets-exec-plan-pueue-task-handle-adapter`

## Required Inputs

- {ref}`knowledge-operations-pueue-integration-runtime-and-validation`
- {ref}`process-assets-work-package-pueue-wait-surface`
- {ref}`process-assets-work-package-pueue-log-handle-surface`
- `docutouch-server/tests/stdio_smoke.rs`
- `docutouch-server/tests/cli_smoke.rs`
- `docs/source/15_process_assets/50_readiness/pueue_subagent_kickoff_readiness.md`

## Deliverables

- runtime / CLI / MCP parity verification；
- docs build verification；
- kickoff readiness page 的 gate closure；
- records sink ready for audit / status / coverage follow-up。

## Dependencies

- wait surface 与 log-handle surface 都已完成；
- 不重开 accepted interface pages，只做实现与验证闭合。

## Owner Type

- mixed
  - executor 可完成验证与 records sink 指认
  - 主代理负责最终审核与用户对接

## Acceptance

- `cargo test -p docutouch-server` 通过；
- `uv run python -m sphinx -b dummy docs/source docs/_build_dummy` 通过；
- readiness gate 进入可启动 subagent 的状态；
- 任何 remaining ambiguity 都被显式写回 records 或 readiness，而不是留在口头里。

## Exit Route

- 结果交给 {ref}`process-assets-handoff-pueue-validation-and-parity`
- records 写入 `30_records/50_audit/`、`30_records/60_status/`、`30_records/70_coverage/`
