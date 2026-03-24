(process-assets-handoffs-contract)=
# 30 Handoffs 作者契约

## 契约范围

本页裁定哪些单 executor 交接对象应进入 `30_handoffs/`。

## Allowed Objects

- agent handoff
- task brief
- implementation brief

## Disallowed Objects

- total execution plan
- multi-package coordination page
- actual status record
- generic task list

## Dependency Discipline

- handoff 页必须显式说明 `Read These First`、`Allowed Edit Surface`、`Disallowed Areas`、`Exact Deliverable` 与 `Verification Criteria`。
- handoff 不得只给目标，不给禁改边界与验收方式。
- 若对象开始承接多个 executor 之间的统筹关系，应迁回 `10_exec_plans/` 或 `20_work_packages/`。
