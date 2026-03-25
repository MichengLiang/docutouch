(process-assets-work-packages-index)=
# 20 Work Packages

## 作用域

本目录承载从 execution plan 拆出的可执行工作包。

它回答：

- 某一段工作到底交付什么；
- 它依赖什么；
- 谁来做；
- 做完之后流向哪里。

## 典型对象

- work package
- task breakdown package
- stream package
- module package

## 不承担的职责

- 不写 total execution plan
- 不写 single-agent handoff
- 不写 pure status aggregation

## Dependency Position

### Upstream Dependencies

- `10_exec_plans/`
- relevant `10_knowledge/`
- relevant `20_deliberation/`

### Downstream Dependents

- `30_handoffs/`
- `40_matrices/`
- `50_readiness/`

### Lateral Cross-References

- `20_deliberation/70_worklists/` 可承接 action-only remainder
- `30_records/` 可承接执行后的事实结果

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 family 容器本身的 operation surface
* - `work_package_template.md`
  - Exported support surface
  - 规定 work package 的最小 section 结构
* - `work_package_minimal_example.md`
  - Process asset page
  - 展示当前体系内的最小合格 work package 写法
* - `apply_splice_baseline_locking_work_package.md`
  - Process asset page
  - `apply_splice` broad implementation 之前的基线锁定工作包
* - `apply_splice_deeper_substrate_extraction_work_package.md`
  - Process asset page
  - `apply_splice` deeper substrate seam 的专项工作包
* - `apply_patch_line_number_assist_baseline_locking_work_package.md`
  - Process asset page
  - `apply_patch` line-number-assisted locking broad implementation 之前的基线锁定工作包
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
work_package_template
work_package_minimal_example
apply_splice_baseline_locking_work_package
apply_splice_deeper_substrate_extraction_work_package
apply_patch_line_number_assist_baseline_locking_work_package
```
