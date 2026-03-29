(knowledge-architecture-index)=
# 50 Architecture

## 作用域

本目录承载 accepted architecture description family。

它回答：当前被接纳的 solution structure 如何被描述。

## 典型对象

- architecture description
- solution strategy
- structural views
- runtime views
- deployment views
- crosscutting concepts

## 不承担的职责

- 不写 accepted decision record
- 不写 API contract
- 不写 operational procedure
- 不写 candidate architecture proposal

## Dependency Position

### Upstream Dependencies

- `10_positioning/`
- `20_problem_space/`
- `30_principles/`
- `40_requirements/`

### Downstream Dependents

- `60_decisions/`
- `70_interfaces/`
- `80_operations/`

### Lateral Cross-References

- `60_decisions/` 可回指 architecture objects 作为决策对象
- `90_reference/` 可收 view index，但不替代 architecture description

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 family 容器本身的 operation surface
* - `apply_patch_diagnostics_spec.md`
  - Source-bearing article
  - 承载 `apply_patch` diagnostics subsystem 的 accepted architecture 与 rendering contract
* - `apply_splice_architecture.md`
  - Source-bearing article
  - 承载 `apply_splice` shared-vs-owned boundary 与 internal substrate posture
* - `cli_adapter_spec.md`
  - Source-bearing article
  - 承载 DocuTouch CLI adapter 的 accepted transport architecture 与 parity boundary
* - `pueue_task_handle_adapter.md`
  - Source-bearing article
  - 承载 Pueue task-handle adapter 的 accepted architecture、placement 与 external-vs-internal boundary
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
apply_patch_diagnostics_spec
apply_splice_architecture
cli_adapter_spec
pueue_task_handle_adapter
```
