(process-assets-exec-plans-index)=
# 10 Exec Plans

## 作用域

本目录承载复杂事项的总执行计划对象。

它回答：

- 整体目标是什么；
- 阶段如何切分；
- 工期与并行策略如何组织；
- 风险、replan 触发条件与验收路径是什么。

## 典型对象

- execution plan
- implementation plan
- stage plan
- milestone plan

## 不承担的职责

- 不写单个 executor 的 handoff
- 不写纯 action queue
- 不写 actual status record

## Dependency Position

### Upstream Dependencies

- `00_meta/`
- `10_knowledge/`
- relevant `20_deliberation/`

### Downstream Dependents

- `20_work_packages/`
- `30_handoffs/`
- `40_matrices/`
- `50_readiness/`

### Lateral Cross-References

- `20_deliberation/70_worklists/` 可承接 action-only remainder
- `30_records/60_status/` 可承接执行后的阶段状态

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 family 容器本身的 operation surface
* - `execution_plan_template.md`
  - Exported support surface
  - 规定 execution plan 的最小 section 结构
* - `execution_plan_minimal_example.md`
  - Process asset page
  - 展示当前体系内的最小合格 execution plan 写法
* - `docutouch_roadmap.md`
  - Process asset page
  - 当前产品演进优先级与非优先项的 canonical roadmap host
* - `ux_hardening_plan.md`
  - Process asset page
  - 当前 UX hardening program 的 canonical host
* - `apply_patch_locking_strategy_blackbox_evaluation_program.md`
  - Process asset page
  - `apply_patch` locking strategy 长窗口比较评估 program host
* - `apply_splice_implementation_plan.md`
  - Process asset page
  - `apply_splice` 实施阶段序列的 canonical host
* - `line_number_alignment_rollout_plan.md`
  - Process asset page
  - 行号对齐专项 rollout host
* - `engineering_quality_wave_20260323_plan.md`
  - Process asset page
  - 2026-03-23 工程质量波次 program host
* - `apply_splice_engineering_hardening_plan.md`
  - Process asset page
  - `apply_splice` 当前工程质量 hardening 波次的 canonical host
* - `apply_splice_implementation_schedule_plan.md`
  - Process asset page
  - `apply_splice` v1 phase/gate 调度计划 host
* - `apply_patch_semantics_hardening_plan.md`
  - Process asset page
  - `apply_patch` semantics / compatibility hardening 的 canonical plan host
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
execution_plan_template
execution_plan_minimal_example
docutouch_roadmap
ux_hardening_plan
apply_patch_locking_strategy_blackbox_evaluation_program
apply_splice_implementation_plan
line_number_alignment_rollout_plan
engineering_quality_wave_20260323_plan
apply_splice_engineering_hardening_plan
apply_splice_implementation_schedule_plan
apply_patch_semantics_hardening_plan
```
