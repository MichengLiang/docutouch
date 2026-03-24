(records-index)=
# 30 Records

## 作用域

本目录承载迁移、处置、审计、变更、状态与覆盖等记录对象。

它记录的是：现实如何变化到现在，而不是当前对象是什么。

## 可发现性原则

`records/` 的价值不应依赖读者未来暴力全量阅读。

它的可发现性至少由两层共同保证：

- records 自身的 inventory / migration / disposition / change 等索引面；
- records 指向 current canonical host、successor 或处置触发对象的显式回指。

`knowledge/` 是否反向回指某条 records，则由该历史路径今天是否仍承担解释、审计或防御负荷决定。

## 二级家族

- `inventory/`
- `migration/`
- `disposition/`
- `change/`
- `audit/`
- `status/`
- `coverage/`

## 内部结构

`30_records/` 内部采用三组结构：

- 入口与盘点层：`inventory/`
- 对象级变化记录层：`migration/`、`disposition/`、`change/`、`audit/`
- 聚合与监控层：`status/`、`coverage/`

## 总体依赖图

```{list-table}
:header-rows: 1

* - 家族
  - 主要上游
  - 主要下游
* - `inventory/`
  - 当前存量对象与材料现实
  - `migration/`、`disposition/`、`coverage/`、`audit/`
* - `migration/`
  - `inventory/`、相关 accepted / unresolved object
  - `disposition/`、`status/`、`coverage/`、current canonical host
* - `disposition/`
  - `inventory/`、`migration/`、resolved object
  - `status/`、`coverage/`、successor / current host
* - `change/`
  - accepted knowledge object
  - `status/`、`coverage/`、current canonical object
* - `audit/`
  - accepted / unresolved object 或 records 自身
  - `20_deliberation/`、`change/`、`disposition/`、`status/`
* - `status/`
  - `migration/`、`disposition/`、`change/`、`audit/`
  - `coverage/`
* - `coverage/`
  - `inventory/`、`migration/`、`disposition/`、`status/`
  - 一般不再作为其他家族的 authority 上游
```

## `migration / disposition / change` 的 canonical boundary

```{list-table}
:header-rows: 1

* - 判断维度
  - `migration/`
  - `disposition/`
  - `change/`
* - 主要回答的问题
  - 对象从哪里到哪里去了
  - 对象最后被怎样判了
  - accepted 对象本身后来怎样变了
* - 强调重点
  - 迁移路径与新宿主
  - 最终判决与对象命运
  - 修订、替代与版本推进
* - 是否期待 successor / current host
  - 通常期待明确目标宿主
  - 若存在 successor，应显式说明
  - 通常应指回 current canonical object 或 superseding object
* - 典型上游触发
  - inventory、吸收、重写、拆分、搬迁
  - reject、supersede、absorb、retire
  - accepted object revision、replacement、baseline update
* - 不应被写成什么
  - 不应写成最终判决页
  - 不应写成迁移路径页
  - 不应写成 inventory 或 accepted object 正文
```

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向 `30_records/` 容器本身的 operation surface
* - `10_inventory/`
  - Family container
  - 入口与盘点层容器
* - `20_migration/`、`30_disposition/`、`40_change/`、`50_audit/`
  - Family container
  - 对象级变化记录层容器
* - `60_status/`、`70_coverage/`
  - Family container
  - 聚合与监控层容器
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
10_inventory/index
20_migration/index
30_disposition/index
40_change/index
50_audit/index
60_status/index
70_coverage/index
```
