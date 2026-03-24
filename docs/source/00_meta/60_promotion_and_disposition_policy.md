(meta-promotion-policy)=
# 升格、降格与处置规则

## 角色

本页说明对象如何在 `knowledge`、`deliberation`、`records` 三个正文对象域之间移动，以及移动的条件是什么。

## 基本对象域

本体系承认三个正文对象域：

- `knowledge`
- `deliberation`
- `records`

它们不是同一条状态轴上的三个值，而是三类不同的正文对象宿主面。

状态变化发生在对象从一个对象域迁移到另一个对象域时；对象域本身不应被重新定义为“状态名”。

对象在三个正文对象域之间移动时，应保留可追溯路径，不得无痕跃迁。

## 与外围过程资产的边界

本页处理的是 `knowledge / deliberation / records` 三域内部对象之间的迁移。

它不直接处理：

- stage 计划文档；
- readiness audit；
- matrix；
- 聊天摘录；
- 其他 build root 内外的 process assets 与 source material。

这些对象如何进入 build-root resident process host、`source authority`、records authority article 或 actual record object，
由 {ref}`meta-process-assets-and-authority-conversion` 继续承担。

## 从 `deliberation` 进入 `knowledge`

一个对象进入 `knowledge/`，通常需要同时满足：

- 它已经完成本轮必要的论证或裁决；
- 它不再依赖未决争议才能成立；
- 下游可以明确依赖它；
- 它在 `knowledge/` 中有清楚的职责宿主。

### 特别规则：candidate specification

candidate specification 不应长期停留在 `deliberation/` 却被下游当成现行知识依赖。

一旦它承担 hand-off 或实现依据，应迁入 `knowledge/` 对应子树，并在原处改写为明确指针或处置说明。

## 从 `knowledge` 进入 `records`

若对象：

- 被替代；
- 不再构成现行知识；
- 仍需保留变更轨迹；

则应迁入 `records/` 对应子树，并留下到新对象或处置页的指向。

## `deliberation` 与 `records` 的边界

- `deliberation/` 记录的是尚未收敛的对象；
- `records/` 记录的是变化事实、处置结果与迁移轨迹。

一个对象若已经不再继续争论，而只剩留痕价值，应退出 `deliberation/`，进入 `records/`。

## `records` 的可发现性规则

`records/` 不能只靠“以后全量阅读时或许会被翻出来”来维持价值。

更健康的规则是：

- `records/` 内部应通过 inventory / migration / disposition / change 等家族维持自足索引；
- 只要存在明确 successor、current canonical host 或处置触发对象，records 应尽量显式回指该对象；
- `knowledge/` 反向回指 `records/` 则应保持选择性，只在某条历史路径仍承担解释、审计或反驳防御负荷时保留。

因此，`records → knowledge` 的可追溯链应尽量强，而 `knowledge → records` 的链接应保持克制。

## `records` 内部的记录类型

### `inventory/`

记录当前遗留材料现实，不判断其现行有效性。

### `migration/`

记录迁移动作与迁移去向。

### `disposition/`

记录保留、删除、吸收、替代等处置结论。

### `change/`

记录已接纳知识对象的显式变更。

### `audit/`

记录审查发现、风险与不一致性。

### `status/`

记录阶段状态与完成情况。

### `coverage/`

记录哪些材料、哪些范围已完成映射与收口。

## 规则与记录的关系

对象域的边界由分类与目录规则裁定；
对象如何在对象域之间迁移，则由根与局部 `authoring_contract` 裁定；
迁移发生后的事实由 `records/` 保存；
records 的可发现性由 records 自身索引与必要的 cross-link 共同保证；
为何采用这种迁移法，则由本页与相关 `00_meta/` 页面说明。
