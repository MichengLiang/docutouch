(process-assets-handoff-pueue-validation-and-parity)=
# Pueue Validation And Parity Handoff

## Task Objective

你是一个全新上下文的子代理，不继承主线程上下文。
你的身份是：**Pueue validation / parity closer**。

你的任务是在 substrate、wait surface、log-handle surface 都已落地后，
统一完成验证、readiness closeout 与 records sink 指认。

## Read These First

- `docs/source/10_knowledge/80_operations/pueue_integration_runtime_and_validation.md`
- `docs/source/15_process_assets/10_exec_plans/pueue_task_handle_adapter_implementation_plan.md`
- `docs/source/15_process_assets/20_work_packages/pueue_validation_and_parity_work_package.md`
- `docs/source/15_process_assets/40_matrices/pueue_integration_execution_matrix.md`
- `docs/source/15_process_assets/50_readiness/pueue_subagent_kickoff_readiness.md`
- `docutouch-server/tests/stdio_smoke.rs`
- `docutouch-server/tests/cli_smoke.rs`

还必须全量阅读已完成的 wait/log-handle 相关改动文件，再开始验证。

## Allowed Edit Surface

- `docutouch-server/tests/stdio_smoke.rs`
- `docutouch-server/tests/cli_smoke.rs`
- `docs/source/15_process_assets/50_readiness/pueue_subagent_kickoff_readiness.md`

## Disallowed Areas

- `docutouch-core/**`
- `docs/source/10_knowledge/**`
- 已由其他 executor 拥有的主要实现文件
- 任何用户弹窗调用
- 任何 forked-context delegation

## Exact Deliverable

- parity / regression verification 结果；
- readiness page 的 gate standing 更新；
- records sink 明确；
- 如发现缺口，truthfully 汇总到 report-back。

## Verification Criteria

- `cargo test -p docutouch-server` 通过；
- docs dummy build 通过；
- readiness gate 能回答“是否可启动下一轮 subagent implementation”；
- 若有失败，失败面被 truthfully 定位，而不是口头掩盖。

## Escalation Conditions

- 核心 tests 失败；
- docs build 失败；
- readiness gate 无法基于现有证据裁定；
- 需要改动其他 executor 正在拥有的实现文件。

## Report-Back Format

- changed files
- commands run
- pass/fail summary
- readiness standing
- record sinks to update

