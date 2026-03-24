(records-coverage-contract)=
# 70 Coverage 作者契约

## 契约范围

本页裁定哪些覆盖情况对象应进入 `coverage/`。

## Allowed Objects

- coverage matrix
- completion coverage view
- unresolved remainder map

## Disallowed Objects

- raw inventory list
- migration detail log
- execution matrix
- accepted object正文

## Dependency Discipline

- coverage 是聚合与监控层，不反向成为对象级 authority source；
- coverage 页应明确自己的输入来源来自哪些 records 家族；
- promotion、accepted revision 与 audit finding 只要改变 mapping completeness，就应触发 coverage refresh；
- 若页主要在做 object-level inventory，应迁回 `10_inventory/`。
