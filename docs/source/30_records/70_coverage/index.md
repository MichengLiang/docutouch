(records-coverage-index)=
# 70 Coverage

## 作用域

本目录承载覆盖情况记录。

它回答：哪些范围、材料、对象家族已经被覆盖，哪些仍未被吸收、裁定或处置。

## 典型对象

- coverage matrix
- source-to-host coverage view
- family-level completion coverage
- unresolved remaining map

## 不承担的职责

- 不做 inventory 明细
- 不做 migration 细节
- 不做 accepted knowledge truth

## Dependency Position

### Upstream Dependencies

- `10_inventory/`
- `20_migration/`
- `30_disposition/`
- `60_status/`

### Downstream Dependents

一般不再作为其他家族的 authority 上游。

### Lateral Cross-References

- `50_audit/` 可用于标出覆盖缺口的审查来源
- `15_process_assets/40_matrices/` 可表达执行面关系，但不代替 coverage 的收口判断

## Exit / Refresh Logic

coverage 通过持续刷新、重算或 supersede 延续，而不升格为 accepted knowledge。

当 promotion、accepted revision 或 audit finding 改变了 mapping completeness，
coverage 应同步刷新，而不应滞后到“以后再统一整理”。

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 family 容器本身的 operation surface
* - `process_assets_rollout_coverage.md`
  - Actual record object
  - 记录 `15_process_assets/` 首轮家族与模板覆盖情况
* - `diagnostics_contract_sync_coverage_20260323.md`
  - Actual record object
  - 记录 2026-03-23 diagnostics contract sync wave 的文档覆盖与剩余 sweep 范围
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
process_assets_rollout_coverage
diagnostics_contract_sync_coverage_20260323
```
