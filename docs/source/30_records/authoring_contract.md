(records-contract)=
# 30 Records 作者契约

## 契约范围

本页裁定：什么对象应作为记录对象保存，而不进入现行知识正文。

## 本层的基本性质

`30_records/` 承载的是变化记录对象，而不是：

- accepted knowledge 本体；
- unresolved deliberation 本体；
- 作者规则本体。

进入本层，意味着对象的主要职责是记录变化、去向、处置、审查、状态或覆盖，而不是承担现行真值。

## 可进入对象

- inventory
- migration note
- disposition note
- change record
- audit finding
- status record
- coverage record

## 不应进入对象

- 现行 accepted knowledge
- 仍在收敛中的议题与提案
- 只规定作者行为的局部维护规则

## Backlink Discipline

- records 应尽量显式指向 current canonical host、successor 或处置触发对象；
- records 自身必须维持可发现性，不依赖知识层的偶然回指；
- `knowledge/` 只有在历史路径仍承担解释、审计或防御负荷时，才选择性回指 `records/`。

## 记录原则

- 记录对象描述变化与处置事实；
- 记录对象不承担现行真值；
- 记录对象应保留到相关现行对象、successor、处置触发对象或处置页的显式指向；
- 若存在 current canonical host，records 应尽量显式回指；
- `knowledge/` 只在历史路径仍承担解释、审计或防御负荷时，才选择性回指 `records/`。

## 聚合层纪律

- `status/` 与 `coverage/` 是聚合与监控层，不应反向定义对象级记录；
- `inventory/` 是入口与盘点层，不应冒充 disposition 或 migration；
- `migration/`、`disposition/`、`change/`、`audit/` 是对象级变化记录层，应尽量保持对象级清晰度。

