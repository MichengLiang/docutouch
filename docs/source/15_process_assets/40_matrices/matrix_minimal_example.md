(process-assets-matrix-example)=
# Matrix Minimal Example

## Scope

本页展示 `15_process_assets` rollout 中 task-to-file / task-to-host 关系的最小合格写法。

## Covered Objects

- root topology integration
- boundary hardening
- records bundle writing

## Matrix

| Relation Type | Source | Target | Status | Note |
| --- | --- | --- | --- | --- |
| updates | root topology integration | `source/index.md` | closed | 根级入口已接纳 `15_process_assets/` |
| updates | root topology integration | `source/authoring_contract.md` | closed | 根契约已新增 process-assets 宿主条件 |
| constrains | boundary hardening | `20_deliberation/70_worklists/` | active | 禁止继续吞并 execution plan / handoff |
| constrains | boundary hardening | `30_records/60_status/` | active | status 不代替 readiness |
| emits-record | records bundle writing | `30_records/40_change/` | planned | 根级 topology change 记录 |

## Boundary

本页只展示 typed relation，
不重写 execution plan 正文，也不重写 actual record 内容。
