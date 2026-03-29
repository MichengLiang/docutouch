(process-assets-handoff-pueue-wait-surface)=
# Pueue Wait Surface Handoff

## Task Objective

你是一个全新上下文的子代理，不继承主线程上下文。
你的身份是：**Pueue wait surface implementer**。

你的任务是落地 `wait_pueue` MCP / CLI surface，
严格对齐 accepted contract，不新增任何 metadata helper tool family。

## Read These First

- `docs/source/10_knowledge/50_architecture/pueue_task_handle_adapter.md`
- `docs/source/10_knowledge/70_interfaces/pueue_wait_and_log_handle_contract.md`
- `docs/source/10_knowledge/80_operations/pueue_integration_runtime_and_validation.md`
- `docs/source/15_process_assets/10_exec_plans/pueue_task_handle_adapter_implementation_plan.md`
- `docs/source/15_process_assets/20_work_packages/pueue_wait_surface_work_package.md`
- `docs/source/15_process_assets/40_matrices/pueue_integration_execution_matrix.md`
- `docutouch-server/src/pueue.rs`
- `docutouch-server/src/tool_service.rs`
- `docutouch-server/src/cli.rs`
- `docutouch-server/src/server.rs`
- `docutouch-server/tests/stdio_smoke.rs`
- `docutouch-server/tests/cli_smoke.rs`

必须全量阅读上述文件后再动手。

## Allowed Edit Surface

- `docutouch-server/src/tool_service.rs`
- `docutouch-server/src/cli.rs`
- `docutouch-server/src/server.rs`
- `docutouch-server/tests/stdio_smoke.rs`
- `docutouch-server/tests/cli_smoke.rs`

## Disallowed Areas

- `docutouch-core/**`
- `docs/source/10_knowledge/**`
- `docutouch-server/src/pueue.rs`（若 substrate 缺口出现，只能升级汇报）
- 任何用户弹窗调用
- 任何 forked-context delegation

## Exact Deliverable

- MCP `wait_pueue` 注册与执行路径；
- CLI `wait-pueue` projection；
- `any` / `all` / timeout / empty snapshot 的 truthful summary surface；
- parity regression coverage。

## Verification Criteria

- 输出字段与 accepted contract 一致；
- CLI / MCP 共用 semantic core；
- 不新增第二个 wait-like / status-like tool；
- `log_handle` 的返回 shape 正确。

## Escalation Conditions

- 需要修改 substrate 文件才能继续；
- 需要改变 `wait_pueue` accepted output grammar；
- 需要新增 popup 或交互式 host capability；
- 无法在 allowed edit surface 内保证 CLI / MCP parity。

## Report-Back Format

- changed files
- scenarios covered
- tests run
- remaining risks
- whether contract drift was encountered

