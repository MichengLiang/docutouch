(process-assets-exec-plans-contract)=
# 10 Exec Plans 作者契约

## 契约范围

本页裁定哪些总执行计划对象应进入 `10_exec_plans/`。

## Allowed Objects

- total execution plan
- stage plan
- milestone plan
- implementation plan
- roadmap / priority plan
- rollout plan
- evaluation program

## Disallowed Objects

- single-agent handoff
- pure task queue
- actual status record
- object-level accepted truth

## Dependency Discipline

- execution plan 页必须显式说明 `Source Basis`、`Milestones And Duration`、`Parallelization Plan` 与 `Acceptance Strategy`。
- execution plan 不得只写 timeline，而缺失输入边界、输出边界与 replan trigger。
- 若对象只剩动作组织职责，应迁到 `20_work_packages/` 或 `20_deliberation/70_worklists/`。
