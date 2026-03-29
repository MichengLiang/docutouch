(process-assets-handoff-pueue-runtime-substrate)=
# Pueue Runtime Substrate Handoff

## Task Objective

你是一个全新上下文的子代理，不继承主线程上下文。
你的身份是：**Pueue runtime substrate implementer**。

你的任务是为 `docutouch-server` 建立 shared Pueue substrate，
但你不是外部 contract 裁判者；遇到边界不清处必须停下上报，不能自行扩 scope。

## Read These First

- `docs/source/10_knowledge/50_architecture/pueue_task_handle_adapter.md`
- `docs/source/10_knowledge/70_interfaces/pueue_wait_and_log_handle_contract.md`
- `docs/source/10_knowledge/80_operations/pueue_integration_runtime_and_validation.md`
- `docs/source/15_process_assets/10_exec_plans/pueue_task_handle_adapter_implementation_plan.md`
- `docs/source/15_process_assets/20_work_packages/pueue_runtime_substrate_work_package.md`
- `docutouch-server/src/main.rs`
- `docutouch-server/src/tool_service.rs`
- `docutouch-server/src/cli.rs`
- `docutouch-server/src/server.rs`
- `docutouch-core/src/fs_tools.rs`

必须全量阅读上述文件后再动手。

## Allowed Edit Surface

- `docutouch-server/src/pueue.rs`
- `docutouch-server/src/main.rs`（仅在需要声明新模块时）
- `docutouch-server/Cargo.toml`（仅在新增依赖不可避免时）

## Disallowed Areas

- `docutouch-core/**`
- `docutouch-server/src/tool_service.rs`
- `docutouch-server/src/cli.rs`
- `docutouch-server/tests/**`
- 任何用户弹窗调用
- 任何 forked-context delegation

## Exact Deliverable

- shared Pueue substrate module；
- runtime / executable / task-log resolution seam；
- active snapshot 与 waiter substrate helper；
- 不重开 accepted contract。

## Verification Criteria

- 代码可被 downstream stream 直接调用；
- 未改写现有 public tool surface；
- 若新增依赖，理由必须在 report-back 中明确说明。

## Escalation Conditions

- 你需要改动 `docutouch-core`；
- 你需要新增第二个外部工具；
- 你无法在 allowed edit surface 内交付 substrate；
- 你认为 accepted docs 无法支撑当前实现。

## Report-Back Format

- changed files
- exported substrate API
- unresolved questions
- tests run
- whether any dependency or env assumption changed

