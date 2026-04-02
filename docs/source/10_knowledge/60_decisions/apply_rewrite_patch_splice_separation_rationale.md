(knowledge-decisions-apply-rewrite-patch-splice-separation-rationale)=
# Apply Rewrite Versus Apply Patch And Apply Splice Separation Rationale

## 作用域

本页记录 `apply_rewrite` 与 `apply_patch`、`apply_splice` 必须保持分立的 accepted rationale。

它回答：

- 为什么 `apply_rewrite` 不是 `apply_patch` 的 hunk 变体；
- 为什么 `apply_rewrite` 不是 `apply_splice` 的 authored-text 扩展；
- 为什么共享底层实现不推出共享 public tool identity。

## Accepted Decision

当前 accepted decision 是：

- `apply_rewrite`、`apply_patch`、`apply_splice` 保持三个独立工具身份；
- `apply_rewrite` 不复用 patch hunk grammar；
- `apply_rewrite` 不复用 splice transfer action basis；
- 能力归属应先问 object boundary，再问实现复用。

## Accepted Rationale

### Why Not `apply_patch`

- `apply_patch` 的核心是 match-first, then replace；
- 它依赖 old-side evidence 与 target snapshot 之间的差异匹配；
- 即便加入 numbered assistance，它仍属于 diff-shaped authoring。

而 `apply_rewrite` 的核心是：

- 先通过 selection block 锁定旧区间；
- 再对这个已锁定旧区间执行 delete 或 replace。

因此它不是 patch grammar 的局部方言，
而是不同的 object model。

### Why Not `apply_splice`

- `apply_splice` 的核心是 source / target span relation；
- 它表达的是 copy、move、delete、append、insert、replace 等 transfer-family 关系；
- 它刻意不在程序内部 author 新文本。

而 `apply_rewrite` 需要：

- 只锁定 old span；
- 再提供 authored replacement payload 或 delete marker；
- 不引入 source-to-target 双侧结构。

因此它不是 splice grammar 上加一个 payload，
而是不同的 semantic family。

### Prompt-Facing Consequence

若把三者混到一个 tool surface：

- 模型会混写 `- old / + new` 与 selection block；
- 模型会把 `Append To File`、`Insert Before` 等 splice 词法借到 rewrite 场景；
- parser 与 diagnostics 会被迫承接混合错误；
- canonical public guidance 会失去窄而硬的稳定性。

高稳定性编辑工具需要的是分立 surface，
不是大而全的统一入口。

## Rejected Alternatives

当前被拒绝的说法包括：

- 把 `apply_rewrite` 作为 `apply_patch` 的 numbered-selection mode；
- 把 `apply_rewrite` 作为 `apply_splice` 的 replace-with-inline-text mode；
- 先按 parser 方便性合并，再靠文档“提醒用户别混用”；
- 用共享 runtime substrate 作为 public identity 合并的直接证据。

## Architectural Consequence

这项裁决允许共享的应是 tool-agnostic substrate，
例如：

- path normalization；
- connected mutation unit grouping；
- staged commit / rollback；
- selection resolution substrate 的通用部分；
- generic diagnostic rendering helpers。

但以下内容必须保持 tool-owned：

- public grammar；
- action basis；
- diagnostics vocabulary；
- prompt-facing docs；
- boundary-specific repair guidance。

## Source Basis

- `temporary/apply_rewrite 正式设计思想与详细规格.md`
- {ref}`knowledge-interfaces-apply-patch-semantics`
- {ref}`knowledge-interfaces-apply-splice-spec`
- {ref}`knowledge-interfaces-apply-rewrite-spec`
