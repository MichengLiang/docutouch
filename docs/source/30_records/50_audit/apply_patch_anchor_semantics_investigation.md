(records-audit-apply-patch-anchor-semantics-investigation)=
# Apply Patch Anchor Semantics Investigation

## Role

本页记录 2026-03-23 对 `apply_patch` 当前 `@@` anchor 机制所做的审查发现。

它回答的是：

- 当前 parser / runtime 实际支持什么；
- 活跃文档面与运行时之间存在什么 drift；
- 哪些更强的解释不应被误当作现行合同。

## Source Artifact

- `docs/temporary/apply_patch_anchor_semantics_investigation.md`

## Record Scope

本记录只保留 investigation 已经支持的 finding。

它不承担：

- 最终 grammar 决策；
- future line-locked extension 的完整设计；
- implementation schedule。

## Findings Summary

当前 investigation 支持的主要 finding 包括：

1. 当前 parser 每个 chunk 最多只接受一个显式 `@@` header；
2. 当前 runtime 将 `@@` 用作单行 coarse pre-anchor，而不是 hierarchy / scope system；
3. active prompt-facing docs 教授 stacked multi-`@@`，但 parser 并不支持该路径；
4. 当前更健康的本地解释是 single-anchor entry，而不是 stacked hierarchy；
5. stacked multi-`@@` 不是当前最可信、最高优先级的投入方向。

## Evidence Basis

- local parser / runtime code
- local injected tool docs
- 2026-03-23 对 upstream public materials 的对照
- cross-model field observation

本记录的价值在于保留“已经成立的审查发现”，而不是继续展开所有推演分支。

## Downstream Effect

本记录直接支撑：

- current `apply_patch` interface-semantics truthfulness work
- prompt/runtime drift cleanup
- later warning / hardening planning

## Boundary

本页是 actual record object。

它不重写：

- current interface semantics 正文；
- accepted rationale；
- future grammar promotion。
