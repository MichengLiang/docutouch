(process-assets-handoffs-index)=
# 30 Handoffs

## 作用域

本目录承载面向单个 executor 的执行交接包。

它回答：

- 这个 executor 应该做什么；
- 先读什么；
- 哪些地方能动；
- 哪些地方不能动；
- 交付物与验证标准是什么。

## 典型对象

- agent handoff
- task brief
- implementation brief

## 不承担的职责

- 不写 total execution plan
- 不写 multi-package coordination
- 不写 actual status record

## Dependency Position

### Upstream Dependencies

- `10_exec_plans/`
- `20_work_packages/`
- relevant `10_knowledge/`

### Downstream Dependents

- human / agent executors
- `30_records/`

### Lateral Cross-References

- `40_matrices/` 可表达 handoff 与文件/责任的关系
- `50_readiness/` 可表达 handoff 完成后的 gate 条件

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 family 容器本身的 operation surface
* - `agent_handoff_template.md`
  - Exported support surface
  - 规定 handoff 的最小 section 结构
* - `agent_handoff_minimal_example.md`
  - Process asset page
  - 展示当前体系内的最小合格 handoff 写法
* - `apply_splice_parser_selection_handoff.md`
  - Process asset page
  - `apply_splice` parser / selection stream 的首个执行交接包
* - `apply_splice_selection_resolution_handoff.md`
  - Process asset page
  - `apply_splice` selection resolution 子流的执行交接包
* - `apply_splice_shared_substrate_handoff.md`
  - Process asset page
  - `apply_splice` shared substrate 子流的执行交接包
* - `apply_splice_runtime_handoff.md`
  - Process asset page
  - `apply_splice` runtime core 子流的执行交接包
* - `apply_splice_presentation_transport_handoff.md`
  - Process asset page
  - `apply_splice` presentation / transport 子流的执行交接包
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
agent_handoff_template
agent_handoff_minimal_example
apply_splice_parser_selection_handoff
apply_splice_selection_resolution_handoff
apply_splice_shared_substrate_handoff
apply_splice_runtime_handoff
apply_splice_presentation_transport_handoff
```
