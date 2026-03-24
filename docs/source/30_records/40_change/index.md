(records-change-index)=
# 40 Change

## 作用域

本目录承载 accepted objects 的变更记录。

它回答：已处于 accepted knowledge 的对象在 canonical identity 连续前提下，后来发生了什么修订、同一 identity 内的替换或版本推进。

## 典型对象

- change record
- revision note
- replacement mapping
- baseline change note
- propagation-review-triggering revision

## 不承担的职责

- 不做 inventory
- 不做 unresolved issue
- 不做 accepted object正文

## Dependency Position

### Upstream Dependencies

- relevant accepted knowledge object

### Downstream Dependents

- `60_status/`
- `70_coverage/`
- current canonical object

### Lateral Cross-References

- `20_migration/` 与 `30_disposition/` 可在对象级变化上互相指认，但不互相替代

## Exit / Refresh Logic

change 记录本身保留；
它应始终把读者带回 current canonical object。

若对象的 canonical identity 已退出当前现行面，
则不应继续停留在 `change/`，
而应 hand-off 到 `disposition/`。

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 family 容器本身的 operation surface
* - `root_topology_process_assets_change.md`
  - Actual record object
  - 记录根级 object-domain partition 的显式变更
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
root_topology_process_assets_change
```
