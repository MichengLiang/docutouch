(process-assets-exec-plan-template)=
# Execution Plan Template

## Role

本页定义 execution plan 的最小 section 结构。

## Fixed Sections

execution plan 默认提供：

- `Purpose`
- `Source Basis`
- `Target Outcome`
- `Scope`
- `Non-Goals`
- `Milestones And Duration`
- `Dependency Strategy`
- `Parallelization Plan`
- `Acceptance Strategy`
- `Risk And Replan Triggers`
- `Related Work Packages`
- `Related Records`

## Required Relation Sections

`Source Basis`、`Related Work Packages` 与 `Related Records`
不是可选装饰 section，
而是 execution plan 在 build root 中保持可追溯性的最低 relation surface。

## Section Duties

### `Purpose`

回答为什么值得推进当前事项。

### `Source Basis`

显式列出 plan 依赖的 accepted knowledge、deliberation object 与外部约束。

### `Milestones And Duration`

表达阶段切分、预计持续时间与阶段性完成条件。

推荐至少提供：

```text
Milestone | Target Outcome | Expected Duration | Entry Condition | Exit Condition
```

### `Parallelization Plan`

表达哪些工作可并行、哪些必须串行，以及并行边界是什么。

### `Acceptance Strategy`

表达整件事项最终如何被验证，而不是把验收散落在口头记忆中。

### `Related Work Packages`

列出从当前总计划拆出的 canonical work package 宿主。

### `Related Records`

列出当前计划推进后最可能写入的 records family 或具体 records page。

## Warning Cases

以下情况应被视为 execution plan 失真：

- 只有 timeline，没有 `Source Basis`
- 只有 milestone，没有 `Acceptance Strategy`
- 只写“做完即可”，没有 `Risk And Replan Triggers`
- 用 handoff 级别的细粒度禁改边界替代整体执行结构

## Boundary

execution plan 不应退化为：

- 纯工期表；
- 纯 action list；
- 单 agent brief；
- actual status record。
