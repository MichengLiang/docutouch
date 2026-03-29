(knowledge-interfaces-index)=
# 70 Interfaces

## 作用域

本目录承载 accepted contract surface family。

它回答：系统对外暴露的接口、协议与 schema 是什么。

## 典型对象

- API contract
- protocol surface
- schema
- integration contract
- consumer contract layer

## 不承担的职责

- 不写 architecture overview
- 不写 accepted decision record
- 不写 candidate interface proposal
- 不写 operational workaround

## Dependency Position

### Upstream Dependencies

- `30_principles/`
- `40_requirements/`
- `50_architecture/`

### Downstream Dependents

- `60_decisions/`
- `80_operations/`

### Lateral Cross-References

- `50_architecture/` 可链接 interfaces 说明对外面，但不应由 interface 反向定义 architecture ontology
- `90_reference/` 可汇总 schema index，但不替代 interface contract

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 family 容器本身的 operation surface
* - `apply_patch_semantics.md`
  - Source-bearing article
  - 记录 `apply_patch` 当前已接纳的 interface semantics 与 warning contract
* - `apply_splice_spec.md`
  - Source-bearing article
  - 记录 `apply_splice` 的稳定 contract surface 与 action basis
* - `read_file_sampled_view_spec.md`
  - Source-bearing article
  - 记录 `read_file` sampled view 的参数、验证与输出 contract
* - `search_text_ux_contract.md`
  - Source-bearing article
  - 记录 `search_text` 的 grouped search UX contract
* - `pueue_wait_and_log_handle_contract.md`
  - Source-bearing article
  - 记录 `wait_pueue` 与 `pueue-log:<id>` handle 的 accepted contract surface
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
apply_patch_semantics
apply_splice_spec
read_file_sampled_view_spec
search_text_ux_contract
pueue_wait_and_log_handle_contract
```
