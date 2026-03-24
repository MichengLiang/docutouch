(deliberation-contract)=
# 20 Deliberation 作者契约

## 契约范围

本页裁定：什么对象可以停留在未收敛空间，以及何时必须退出。

## 本层的基本性质

`20_deliberation/` 承载的是尚未接纳、因而不可被下游依赖的对象。

因此：

- 进入本层，不表示对象“无价值”；
- 进入本层，表示对象仍需进一步论证、裁决、担保、补证或处置；
- 本层对象不得反向成为 `10_knowledge/` 的 authority source。

## 可进入对象

- issue
- proposal
- assumption
- candidate specification
- conflict
- evidence gap
- worklist

## 不应进入对象

- 已经成为现行依赖前提的知识对象
- 只剩留痕价值、不再继续收敛的历史对象
- 只规定作者行为的局部规则

## 锚定规则

除非对象明确属于“关于框架自身的未决问题”，否则每个 deliberation 对象都应显式锚定到：

- 某个 accepted knowledge family；或
- 某个具体 accepted object；或
- 某条已有 deliberation 主干对象。

不允许让 deliberation 对象长期悬浮而没有 target surface。

## 退出条件

- 若对象被接纳为现行知识，迁入 `10_knowledge/`
- 若对象不再继续收敛，只剩变化与处置价值，迁入 `30_records/`

## Dependency Discipline

- `issues/` 是主干入口，但不应吞并 proposal、conflict、gap 等对象
- `candidate_specs/` 是主干末端，不等同于普通 proposal
- `assumptions/` 是桥接层，不等同于 accepted principles
- `worklists/` 是派生支撑层，不得反向统治其他家族
- 普通 cross-reference 不自动构成 authority dependency

