(process-assets-handoff-pueue-log-handle-surface)=
# Pueue Log-Handle Surface Handoff

## Task Objective

你是一个全新上下文的子代理，不继承主线程上下文。
你的身份是：**Pueue log-handle surface implementer**。

你的任务是让 `read_file` / `search_text` 接受 `pueue-log:<id>`，
同时保持既有 content-first 与 grouped-search contract 不漂移。

## Read These First

- `docs/source/10_knowledge/50_architecture/pueue_task_handle_adapter.md`
- `docs/source/10_knowledge/70_interfaces/pueue_wait_and_log_handle_contract.md`
- `docs/source/10_knowledge/70_interfaces/read_file_sampled_view_spec.md`
- `docs/source/10_knowledge/70_interfaces/search_text_ux_contract.md`
- `docs/source/10_knowledge/80_operations/pueue_integration_runtime_and_validation.md`
- `docs/source/15_process_assets/10_exec_plans/pueue_task_handle_adapter_implementation_plan.md`
- `docs/source/15_process_assets/20_work_packages/pueue_log_handle_surface_work_package.md`
- `docs/source/15_process_assets/40_matrices/pueue_integration_execution_matrix.md`
- `docutouch-server/src/pueue.rs`
- `docutouch-server/src/tool_service.rs`
- `docutouch-server/src/cli.rs`
- `docutouch-server/tests/stdio_smoke.rs`
- `docutouch-server/tests/cli_smoke.rs`

必须全量阅读上述文件后再动手。

## Allowed Edit Surface

- `docutouch-server/src/tool_service.rs`
- `docutouch-server/src/cli.rs`
- `docutouch-server/tests/stdio_smoke.rs`
- `docutouch-server/tests/cli_smoke.rs`

## Disallowed Areas

- `docutouch-core/**`
- `docs/source/10_knowledge/**`
- `docutouch-server/src/pueue.rs`（若 substrate 缺口出现，只能升级汇报）
- 任何用户弹窗调用
- 任何 forked-context delegation

## Exact Deliverable

- `read_file` 的 handle branch；
- `search_text` 的 handle branch；
- `task missing` vs `log missing` 的 truthful differentiation；
- 不新增 metadata header 的 read regression；
- 不新增第二套 Pueue-only search UX。

## Verification Criteria

- `read_file` 成功时仍只返回内容本身；
- `search_text` 成功时仍保持 grouped-by-file 发现面；
- CLI / MCP 两个 transport 上的 handle 行为一致；
- 错误文本 truthfully 区分 task missing / log missing。

## Escalation Conditions

- 你需要改 `docutouch-core` 才能支持 handle branch；
- 你需要给 `read_file` 增加新的 metadata header；
- 你认为 `pueue-log:<id>` grammar 本身需要重开；
- 你无法在 allowed edit surface 内完成 parity。

## Report-Back Format

- changed files
- handle cases covered
- error cases covered
- tests run
- any remaining ambiguity

