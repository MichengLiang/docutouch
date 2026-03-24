(process-assets-readiness-plan-template)=
# Readiness Plan Template

## Role

本页定义 readiness plan 的最小 section 结构。

## Fixed Sections

readiness plan 默认提供：

- `Readiness Scope`
- `Target Gate`
- `Required Inputs`
- `Open Risks`
- `Entry Conditions`
- `Exit Conditions`
- `Related Status Records`

## Recommended Gate Table

readiness page 推荐至少提供：

```text
Gate Item | Current Standing | Blocking Risk | Required Action | Record Sink
```

## Section Duties

### `Target Gate`

表达当前 readiness page 面向哪个明确 gate。

### `Required Inputs`

表达 gate 前必须具备的输入对象。

### `Open Risks`

表达仍未闭合的风险，而不是把风险隐藏在叙述性 prose 中。

### `Related Status Records`

显式回指 gate 闭合后的 records host。

## Warning Cases

以下情况应被视为 readiness plan 失真：

- 把 audit finding 直接写成 readiness 事实
- 只有 checklist，没有目标 gate
- 只有 open risks，没有 exit conditions
- 不回指后续 status host

## Boundary

readiness plan 不应退化为：

- actual audit report；
- generic status row；
- total execution plan。
