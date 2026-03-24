(records-change-contract)=
# 40 Change 作者契约

## 契约范围

本页裁定哪些 accepted object 的变更事件应进入 `change/`。

## Allowed Objects

- revision record
- replacement note
- baseline change record
- propagation-review-triggering revision

## Disallowed Objects

- migration-only note
- unresolved discussion
- accepted object正文

## Dependency Discipline

- change 页必须指回 current canonical object；
- 若 revision 可能影响 dependents，应同步 hand-off 到 `status/` 与 `coverage/`，表达 propagation review 是否闭合；
- 若页主要在记录对象被搬到哪里，应迁入 `20_migration/`；
- 若页主要在记录对象最终被如何判定，应迁入 `30_disposition/`。
