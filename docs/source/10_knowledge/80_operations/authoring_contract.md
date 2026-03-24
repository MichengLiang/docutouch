(knowledge-operations-contract)=
# 80 Operations 作者契约

## 契约范围

本页裁定 accepted operational knowledge objects 的落位边界。

## Allowed Objects

- configuration fact
- operational procedure
- maintenance procedure
- lifecycle operation knowledge
- recovery / export / backup knowledge

## Disallowed Objects

- architecture description
- accepted decision record
- candidate workaround
- migration / audit record

## Dependency Discipline

- operations 以前置 principles、requirements、architecture 与 interfaces 为上游
- operations 应被视为 accepted knowledge tree 的下游消费面，而不是上游统治面
- 若对象主要记录的是变化而非 current operation knowledge，应迁入 `30_records/`

