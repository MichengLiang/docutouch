(deliberation-worklists-contract)=
# 70 Worklists 作者契约

## 契约范围

本页裁定哪些行动组织对象应进入 `worklists/`。

## Allowed Objects

- action list
- next-step queue
- resolution checklist
- review queue

## Disallowed Objects

- issue / proposal / assumption / candidate spec / evidence gap 的对象定义
- total execution plan
- agent handoff
- task matrix
- readiness plan
- migration record
- accepted knowledge statement

## Dependency Discipline

- worklist 页必须显式指向其来源 deliberation 对象
- worklists 是派生支撑层，不得反向成为其他家族的 authority source
- 若某页开始承担对象定义职责，应迁回相应主体家族
