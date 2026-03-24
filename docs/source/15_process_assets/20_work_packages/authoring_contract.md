(process-assets-work-packages-contract)=
# 20 Work Packages 作者契约

## 契约范围

本页裁定哪些可执行工作包对象应进入 `20_work_packages/`。

## Allowed Objects

- work package
- task breakdown package
- stream package
- module package

## Disallowed Objects

- total execution plan
- single-agent handoff
- actual status record
- generic next-step list

## Dependency Discipline

- work package 页必须显式说明 `Required Inputs`、`Deliverables`、`Dependencies`、`Owner Type` 与 `Exit Route`。
- work package 不得只有动作列表，而没有交付物定义与验收条件。
- 若对象已经压缩到单个 executor 的禁改边界与交付格式，应迁入 `30_handoffs/`。
