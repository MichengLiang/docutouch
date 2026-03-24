(process-assets-work-package-template)=
# Work Package Template

## Role

本页定义 work package 的最小 section 结构。

## Fixed Sections

work package 默认提供：

- `Objective`
- `Upstream Plan`
- `Required Inputs`
- `Deliverables`
- `Dependencies`
- `Owner Type`
- `Acceptance`
- `Exit Route`

## Required Relation Sections

`Upstream Plan`、`Required Inputs` 与 `Exit Route`
共同构成 work package 的最小 relation surface。

## Section Duties

### `Required Inputs`

列出本工作包必须依赖的文件、页面、知识对象或 records context。

### `Deliverables`

列出当前工作包完成后必须出现的对象，而不是只写“完成此任务”。

建议显式区分：

- primary deliverable
- supporting update
- expected record sink

### `Owner Type`

表达该包由 human、agent 或 mixed 承担。

### `Exit Route`

表达完成后的结果回写到哪里。

## Warning Cases

以下情况应被视为 work package 失真：

- 只有 action item，没有 deliverable
- 没有 `Required Inputs`
- 没有 `Owner Type`
- 把单 executor 的禁改边界写进 work package，而不下放到 handoff

## Boundary

work package 不应退化为：

- total execution plan；
- 单 agent brief；
- pure status row。
