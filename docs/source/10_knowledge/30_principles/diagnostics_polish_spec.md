(knowledge-principles-diagnostics-polish-spec)=
# Diagnostics Polish Principles

## Role

本页承载 diagnostics polishing 的 accepted doctrine 与 judgment rubric。

它回答的是：

- 什么样的 diagnostics polish 值得被接纳；
- 什么样的“改进”其实应被拒绝；
- tool-layer diagnostics 在 truth、density、host boundary 上的长期约束是什么。

## Accepted Scope

本页处理的是长期有效的 judgment rule，而不是：

- 某一轮 polish execution plan；
- 候选改进的排期；
- historical workspace 说明；
- 某个具体 patch failure 的局部处置记录。

## North Star

diagnostics subsystem 的 north star 不是“更像编译器”，而是：

- fewer wasted tokens
- fewer wasted repair turns
- stronger truthful blame signals

因此，polish 的目标应始终是更强的 repairability，而不是更强的 aesthetic sophistication。

## Core Doctrine

### 1. Repair-Turn Priority

任何 diagnostics polish 若不能减少模型的 wasted repair turns，默认不值得做。

这条原则优先于：

- 外观统一
- 文案更顺
- 结构更像某个熟悉工具

### 2. Information-Density Priority

改进必须提升 useful information per token，或至少在不损失密度的前提下简化 contract。

若某个改动主要增加：

- prose
- repetition
- decorative structure

则应拒绝。

### 3. Truthfulness Over Style

runtime 只应说自己真正知道的事。

因此 diagnostics 不得依赖：

- fake precision
- speculative blame
- misleading summary language

“更漂亮”从来不是伪造 blame location 的理由。

### 4. Contract Unification With Meaningful Distinctions Preserved

原则上应减少不必要 special cases，但不能为了统一而抹平有意义的结构差异。

典型例子是：

- single full failure 应保持 compact；
- partial success 必须保留更重的 repair accounting。

这里允许不对称，只要这种不对称来自对象职责，而不是历史偶然。

### 5. Maintenance Sanity

任何 polish 若引入 naming churn、taxonomy churn 或 architecture churn，却没有 durable contract benefit，应视为不值得做。

polish 不是为了制造持续整理工作。

### 6. Host-Boundary Discipline

tool layer 的职责是 repair-facing diagnostics，而不是 durable audit theater。

因此：

- repair-relevant inline accounting 是允许的；
- failed patch source persistence 在确有 repair value 时是允许的；
- second-layer audit subsystem、replay cache、secondary JSON reporting 不属于 tool-layer polish 目标。

### 7. Common-Path Discipline

ordinary path 必须保持 boring。

single full failure 默认应是 compact path。

只有当额外结构明确改变 next repair move 时，才值得扩展 common path。

## Reusable Gate Set

任何候选 diagnostics polish 都应经过以下 gate set：

1. `repair-turn gain`
2. `information-density gain`
3. `truthfulness`
4. `contract unification`
5. `maintenance sanity`
6. `host-boundary discipline`
7. `common-path discipline`

accepted judgment rule 是：

- 只有明确通过 Gates 1, 2, 3，且不违反 4 到 7，才值得关闭；
- 若只对窄场景成立，应以 selective scope 接纳，而不是推广为普遍 polish；
- 若主要收益是 aesthetic neatness，则不应接纳。

## No-Reopen Boundaries

以下“看起来不完美”的点，不应仅因审美压力而重开：

- partial success 比 single full failure 更重
- 某些 low-level causes 保留原始感
- 不是每个 failure 都值得 multi-span
- failure classes 之间存在少量 asymmetry

这些现象只要仍服务于 truthful repair loop，就不构成 reopen trigger。

## Ideal Contract Commitments

diagnostics polish 的长期目标应持续约束为：

- stable code-bearing headline
- smallest truthful blame location
- cause 行提供真实新增信息
- `help:` 只表达最小 next repair move
- partial-failure accounting 完整可见
- tool layer 不生长为 host audit subsystem

## What Principles Govern Candidate Selection

以下方向通常值得继续推进：

- source-span-grade execution diagnostics
- 在强证据下增加一个 target-side anchor
- systematize diagnostics family without inflating the common path
- durable verification for high-value failure classes

以下方向默认应保持克制：

- broad multi-span rendering by default
- decorative symmetry-only cleanup
- 只为“更像 rustc”而做的结构扩张
- 将历史工作区残留包装成长期 doctrine

## Downstream Role

本页作为 principles object，可被以下家族复用：

- `50_architecture/`：约束 diagnostics subsystem architecture
- `70_interfaces/`：约束 public failure surface wording
- `80_operations/`：约束维护与验证策略
- `60_decisions/`：在局部议题中引用其 doctrine 作为裁决依据

## Non-Goals

- 不记录某一轮 polish 已做完什么；
- 不裁定某次具体 implementation wave 的顺序；
- 不保存临时 workspace 的 existence proof；
- 不把 candidate register 原样保留为长期 principles page。
