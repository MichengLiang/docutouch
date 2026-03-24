(process-assets-matrices-index)=
# 40 Matrices

## 作用域

本目录承载 execution-facing relation matrices。

它回答：

- task 与 file、owner、dependency、agent 之间的关系如何被正式表达；
- 哪些责任、覆盖与边界需要矩阵而不是散文。

## 典型对象

- task-to-file matrix
- ownership matrix
- dependency matrix
- agent assignment matrix

## 不承担的职责

- 不写长篇 plan prose
- 不写 single-agent brief
- 不写 actual status record

## Dependency Position

### Upstream Dependencies

- `10_exec_plans/`
- `20_work_packages/`
- `30_handoffs/`

### Downstream Dependents

- executors
- `50_readiness/`
- `30_records/70_coverage/`

### Lateral Cross-References

- `10_knowledge/90_reference/` 可提供 relation grammar 的外层参照

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 family 容器本身的 operation surface
* - `matrix_surface_grammar.md`
  - Exported support surface
  - 规定 execution-facing matrix 的最小字段集
* - `matrix_minimal_example.md`
  - Process asset page
  - 展示当前体系内的最小合格 matrix 写法
* - `apply_splice_implementation_stream_matrix.md`
  - Process asset page
  - `apply_splice` 第一批 implementation streams 的关系矩阵
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
matrix_surface_grammar
matrix_minimal_example
apply_splice_implementation_stream_matrix
```
