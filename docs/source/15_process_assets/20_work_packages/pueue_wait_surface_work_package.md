(process-assets-work-package-pueue-wait-surface)=
# Pueue Wait Surface Work Package

## Objective

在不扩张外部工具面的前提下，落地唯一新增 external tool：`wait_pueue`，
并提供与 CLI `wait-pueue` 一一对应的 transport projection。

## Upstream Plan

- {ref}`process-assets-exec-plan-pueue-task-handle-adapter`

## Required Inputs

- {ref}`knowledge-interfaces-pueue-wait-and-log-handle-contract`
- {ref}`knowledge-operations-pueue-integration-runtime-and-validation`
- {ref}`process-assets-work-package-pueue-runtime-substrate`
- `docutouch-server/src/tool_service.rs`
- `docutouch-server/src/server.rs`
- `docutouch-server/src/cli.rs`
- `docutouch-server/tests/stdio_smoke.rs`
- `docutouch-server/tests/cli_smoke.rs`

## Deliverables

- MCP `wait_pueue` tool registration 与 transport wiring；
- CLI `wait-pueue` projection；
- `any` / `all` / timeout / empty snapshot 的 truthful output surface；
- 对应的 server/CLI parity tests。

## Dependencies

- Pueue substrate 已提供 runtime resolution 与 waiter seam；
- 不得把 metadata helper 升级为新的 tool family。

## Owner Type

- agent

## Acceptance

- `wait_pueue` output grammar 与 accepted contract 对齐；
- CLI / MCP 共享同一 semantic core；
- `current_time`、`reason`、`resolved_task_ids`、`log_handle` 等字段 shape 稳定；
- 不引入第二个 wait-like 或 status-like 工具。

## Exit Route

- 结果交给 {ref}`process-assets-handoff-pueue-wait-surface`
- completion status 写入 `30_records/60_status/`
