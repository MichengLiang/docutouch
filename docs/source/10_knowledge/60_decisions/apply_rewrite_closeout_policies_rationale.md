(knowledge-decisions-apply-rewrite-closeout-policies-rationale)=
# Apply Rewrite Closeout Policies Rationale

## 作用域

本页记录 `apply_rewrite` 当前批次已经拍板的 closeout policy。

它固定的是已经接受、必须同步回写到 interface、architecture 与 tool docs 的执行面裁决，
不是候选方向，也不是待讨论事项。

## Accepted Decision

当前 accepted decision 是：

- `move + rewrite` 的 success summary 保持 final-path accounting：`A destination` + `D source`；
- `WithBlock` 的结果组合必须执行 result-side line-boundary normalization，防止 replacement payload 与保留 suffix 发生灾难性同线拼接；
- `Delete File` 命中不存在路径时按硬失败处理，并给出 repair-first guidance；
- overwrite behavior 只作为 compatibility reality 披露，不提升为 canonical authoring，并且成功面可伴随 warning。

## Accepted Rationale

### Success Summary Must Stay Path-Truthful

`move + rewrite` 对文件系统的最终可见结果是：

- destination 获得新结果；
- source 路径消失。

因此 success summary 必须保持 `A destination` + `D source` 的 final-path accounting。

把这类结果压缩成抽象 `M` 会隐藏 source disappearance，
削弱 partial success、repair 判断与审计可读性。

### `WithBlock` Needs Explicit Result-Side Newline Policy

rewrite payload 是 authored text，
但结果文件的边界完整性仍由 runtime 负责。

当 replacement payload 无终止换行、其后仍有保留 suffix 时，
若简单按字节直连，runtime 会把原本应分隔的两行拼成一行。

因此 accepted contract 必须要求 result-side line-boundary normalization：

- 有 suffix 时优先维持结果行边界；
- 替到 EOF 且无 suffix 时，保留 authored EOF 状态。

这不是未来优化，而是当前 contract 的组成部分。

### Missing Delete Target Is A State Error

`Delete File` 的对象是既有文件。

当目标路径不存在时，调用者对当前 workspace state 的判断已失真，
因此应按 hard failure 处理，而不是静默成功或弱提示跳过。

repair guidance 也必须明确：

- 先重新读取 workspace；
- 确认文件是否已被移除；
- 若确已不存在，则从程序中去掉该 delete。

### Overwrite Reality Must Not Be Mistaught

当前 runtime 允许某些 overwrite-tolerant behavior，
这是 compatibility reality，不是 canonical authoring。

正式文档必须同时表达两件事：

- reality 可以被披露；
- canonical advice 仍然是 `Add File` 用于 true creation、`Move to` 用于 true rename。

当 overwrite reality 实际发生时，success surface 可追加 warning。
这能在不歪曲 primary success contract 的前提下保留 repair signal。

## Consequence

这项裁决要求以下对象保持一致：

- `docutouch-server/tool_docs/apply_rewrite.md`
- {ref}`knowledge-interfaces-apply-rewrite-spec`
- {ref}`knowledge-architecture-apply-rewrite-architecture`

若实现与这些规则不一致，应视为代码侧待对齐事项，
而不是把文档重新写回开放状态。

## Source Basis

- `temporary/apply_rewrite 正式设计思想与详细规格.md`
- `docutouch-server/tool_docs/apply_rewrite.md`
- {ref}`knowledge-interfaces-apply-rewrite-spec`
- {ref}`knowledge-architecture-apply-rewrite-architecture`
