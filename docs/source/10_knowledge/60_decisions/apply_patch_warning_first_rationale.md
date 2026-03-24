(knowledge-decisions-apply-patch-warning-first-rationale)=
# Apply Patch Warning-First Rationale

## 作用域

本页记录 `apply_patch` 在 overwrite-tolerant compatibility 行为上的已接纳裁决理据。

它回答的是：

- 为什么当前不立即把 `Add File` / `Move to` overwrite 行为收紧成默认错误；
- 为什么应采用 warning-first，而不是 doc-only disclosure 或直接 strict-first；
- 这一裁决对下游 interface、runtime 与 hardening priority 有什么后果。

## Decision

当前 accepted decision 为：

- 暂不把 general `Add File` overwrite 与 `Move to` overwrite 收紧为默认错误；
- 将这些行为视为 compatibility-tolerated runtime behavior，而不是 preferred authoring path；
- 在行为实际发生时追加 triggered warning；
- 保留 core/Codex success summary shape，不把 warning 嵌入为新的 primary success contract。

## Authority Basis

- public upstream fixtures 已显式覆盖 overwrite-tolerant 行为；
- 当前 consumer 是 LLM，silent success 会固化错误策略；
- 当前更紧迫的风险位于 path identity、Windows path semantics 与 same-path move correctness；
- 文档披露单独存在仍不足以在触发点修正策略。

## Alternatives Considered

### Alternative A: 立即改为 strict-first

不接受。

理由：

- 会对已存在的上游兼容面形成硬 break；
- 会增加 agent/tool-use 分布摩擦；
- 当前没有足够 evidence 证明必须立即用默认错误替代 warning-first。

### Alternative B: 仅靠文档披露

不接受。

理由：

- runtime 行为仍会在成功路径中静默发生；
- 文档的纠偏力度弱于 trigger-time feedback；
- 这会继续让模型把 tolerated behavior 误学成 preferred tactic。

### Alternative C: 改写 success summary 作为主提示面

不接受。

理由：

- 会破坏 core/Codex common-path success contract；
- 会削弱 `A/M/D` operation accounting 的稳定性；
- warning block 追加已足以提供策略纠偏，而不需要改写 primary success surface。

## Accepted Consequences

- interface contract 必须显式披露 runtime reality 与 preferred authoring path 的区分；
- server/runtime 应在 overwrite-tolerant 行为实际触发时输出 warning；
- lower-layer correctness hardening 继续优先于 overwrite semantics 的全局收紧；
- 若后续 evidence 显示 warning-first 仍持续鼓励 misuse，应重新评估 strict policy。

## Boundary

本页是 case-like accepted rationale。

它不承担：

- `apply_patch` interface semantics 正文；
- path identity / Windows hardening 的执行计划；
- confirmed gap audit 或 repair wave status 记录。

这些对象分别进入：

- {ref}`knowledge-interfaces-apply-patch-semantics`
- {ref}`process-assets-apply-patch-semantics-hardening-plan`
- {ref}`records-audit-apply-patch-confirmed-gaps-20260323`
- {ref}`records-status-apply-patch-contract-repair-wave-20260323`
