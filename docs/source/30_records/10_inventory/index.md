(records-inventory-index)=
# 10 Inventory

## 作用域

本目录承载 records tree 的入口与盘点对象。

它回答：当前手头到底有哪些对象、材料、页面或文件需要被记录与治理。

## 典型对象

- corpus inventory
- legacy material inventory
- object manifest
- preliminary classification inventory

## 不承担的职责

- 不做迁移说明
- 不做最终处置判断
- 不做现行知识定义
- 不做未收敛对象正文

## Dependency Position

### Upstream Dependencies

无 records 内部上游；它是入口与盘点层。

### Downstream Dependents

- `20_migration/`
- `30_disposition/`
- `50_audit/`
- `70_coverage/`

### Lateral Cross-References

- `60_status/` 可在阶段层面引用 inventory 总量或阶段盘点信息

## Exit / Refresh Logic

inventory 条目本身通常不升格为 knowledge；
它的去向由 migration、disposition、coverage 等对象继续接管。

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 family 容器本身的 operation surface
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
```
