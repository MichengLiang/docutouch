(records-status-contract)=
# 60 Status 作者契约

## 契约范围

本页裁定哪些阶段与完成状态对象应进入 `status/`。

## Allowed Objects

- phase state
- milestone state
- gate state
- completion state
- propagation review state

## Disallowed Objects

- object-level accepted truth
- issue / proposal / candidate spec 正文
- readiness plan
- migration 细节页

## Dependency Discipline

- status 是聚合层，不应反向定义对象级记录；
- 若 accepted revision 已触发 propagation review，status 应显式表达相应 gate 是否闭合；
- 若页主要记录对象级变化，应迁回 `migration/`、`disposition/` 或 `change/`；
- status 应尽量汇总结果，而不重新展开原因链。
