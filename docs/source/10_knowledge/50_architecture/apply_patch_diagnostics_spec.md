(knowledge-architecture-apply-patch-diagnostics-spec)=
# `apply_patch` Diagnostics Architecture

## Role

本页承载 `apply_patch` diagnostics subsystem 的 accepted architecture。

它回答的是：

- diagnostics contract 由哪些层组成；
- 哪些信息属于 primary failure surface；
- patch-side 与 target-side evidence 如何分工；
- success warning 与 failure diagnostics 如何共处于同一产品体系。

## Accepted Boundary

本页处理的是 diagnostics architecture，而不是：

- compatibility rationale；
- warning-first 的 accepted decision record；
- future hardening schedule；
- historical gap audit。

这些内容分别应由 `60_decisions/`、`15_process_assets/` 与 `30_records/` 承担。

## Architectural Goal

`apply_patch` diagnostics 的目标不是模拟编译器外观，而是为失败后的下一轮修复提供最短、最真实的 repair path。

因此，本 subsystem 的 accepted north star 是：

> truthful, compact, repair-oriented diagnostics.

## Core Architecture

### 1. Stable Headline Layer

每个 failure 都应以稳定 code-bearing headline 开始：

```text
error[CODE]: summary
```

这一层回答：

- failure class 是什么；
- failure 是 full failure 还是 partial failure；
- 调用者应以哪一类 repair strategy 理解后续内容。

### 2. Truthful Primary Blame Location

系统应优先暴露最小且真实的 primary blame location。

优先级固定为：

1. exact patch-source span
2. target path when no truthful patch-source span exists
3. patch path only when neither of the above is known

架构约束不是“所有失败都必须有 span”，而是：

- runtime 真知道时必须给出；
- runtime 不知道时不得伪造。

### 3. Minimal Evidence Block

diagnostics 可以给出有限的、用于证明诊断成立的证据块，但不应把 failure surface 变成审计报告。

允许的 evidence 形态包括：

- 一个 patch-side excerpt；
- 一个可选的 target-side anchor；
- 必要时的 action / hunk identity。

其职责是证明 cause，而不是复述整个执行过程。

### 4. Repair Guidance Layer

failure surface 的末端应承载 repair-relevant accounting：

- committed changes, when any were applied
- failed file groups, when relevant
- attempted changes, when relevant
- full committed / failed path enumeration when that accounting changes the next move
- compact `help:` guidance

普通 repair loop 不应依赖第二份 audit artifact 才能继续。

## Partial-Failure Architecture

partial failure 不是异常附录，而是 diagnostics architecture 的 first-class branch。

它必须同时保留：

- what committed
- what failed
- what was attempted

因此 partial-failure path 可以比 single full failure 更重，但这份重量必须来自 repair accounting，而不是解释性 prose。

## Warning And Error Family

warning 与 error 属于同一 diagnostics family，而不是两套彼此偶然相像的 rendering。

共享属性应包括：

- stable code-bearing presentation
- compact wording
- strategy-shaping guidance
- truthful scope discipline

但 warning 不能通过改写 success summary 来实现。

accepted architecture 是：

- 保持 Codex-compatible success summary shape；
- 在触发时追加独立 warning block；
- 不把特殊兼容行为写成推荐性教程。

## Target-Side Anchoring Rule

target-side anchor 是可选增强，而不是默认多 span 渲染。

只有在以下条件同时成立时才应出现：

- causal chain 足够强；
- target-side line 明显改善下一步 repair；
- 额外一行不会把 common path 变成 verbosity path。

accepted rule 是：

- default to one primary blame span
- add one secondary target anchor only when it materially improves repairability

## Host Boundary

durable audit 属于 host，不属于 tool-layer diagnostics architecture。

因此 `apply_patch` tool layer 可以：

- 保留 repair-relevant inline accounting；
- 在原 patch 不是 file-backed 时持久化 failed patch source；

被持久化到 `.docutouch/failed-patches/` 的 failed patch source，
其 accepted architecture 身份是 repair artifact，而不是第二层 audit cache。

因此当该 artifact 以后续 file-backed source 形式重新进入受支持 transport 时：

- 诊断仍应把该 patch file 视为 truthful source；
- transport 可以从 artifact 自身路径恢复其所属 workspace anchor；
- 这条恢复能力的职责是缩短 repair loop，而不是引入额外的 hidden workspace state；

但不应扩张为：

- replay cache subsystem
- audit-shaped secondary report
- second JSON reporting layer for ordinary repair

## Architectural Constraints

- diagnostics precision 不得以 fake sophistication 为代价；
- common path 必须保持 boring and stable；
- richer evidence 只在真能改变 next repair move 时才值得加入；
- presentation improvements 不得反向破坏 runtime truth model。

## Downstream Implications

本页作为 architecture object，可被以下家族依赖：

- `70_interfaces/` 用于界定 public diagnostics surface；
- `80_operations/` 用于界定维护与验证职责；
- `60_decisions/` 用于记录局部 accepted rationale；
- `15_process_assets/` 用于组织后续 hardening sequence。

## Non-Goals

- 不重写 warning-first product rationale；
- 不规定下一波实现排期；
- 不保存 historical regression narrative；
- 不把 every execution failure 都升级为 complex multi-span rendering。
