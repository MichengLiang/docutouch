(knowledge-operations-index)=
# 80 Operations

## 作用域

本目录承载 accepted operational knowledge family。

它回答：系统当前如何被配置、运行、维护、恢复与持续操作。

## 典型对象

- configuration facts
- runbook-like procedures
- maintenance procedures
- lifecycle operation knowledge
- export / backup / recovery knowledge

## 不承担的职责

- 不写 architecture description
- 不写 accepted decision record
- 不写 candidate workaround
- 不写 migration record

## Dependency Position

### Upstream Dependencies

- `30_principles/`
- `40_requirements/`
- `50_architecture/`
- `70_interfaces/`

### Downstream Dependents

通常处于 accepted knowledge tree 较下游位置。

### Lateral Cross-References

- `60_decisions/` 可回指 operational decisions
- `90_reference/` 可汇总 operational reference table，但不替代 operations 本体

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 family 容器本身的 operation surface
* - `maintenance_priorities.md`
  - Source-bearing article
  - 记录维护排序、主路径优先级与文档同步义务
* - `upstream_sync_and_compatibility.md`
  - Source-bearing article
  - 记录上游同步基线、兼容性披露与本地分叉纪律
* - `testing_and_tool_admission.md`
  - Source-bearing article
  - 记录测试分层、回归义务与新增工具准入标准
* - `pueue_integration_runtime_and_validation.md`
  - Source-bearing article
  - 记录 Pueue integration 的 runtime assumptions、configuration facts 与 validation obligations
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
maintenance_priorities
upstream_sync_and_compatibility
testing_and_tool_admission
pueue_integration_runtime_and_validation
```
