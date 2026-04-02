(knowledge-decisions-apply-rewrite-identity-rationale)=
# Apply Rewrite Identity Rationale

## 作用域

本页记录 `apply_rewrite` 作为第三编辑工具身份存在的 accepted rationale。

它回答的不是如何使用，
而是对象边界层面的已接纳裁决：

- 为什么 `apply_rewrite` 不是已有工具的小扩展；
- 为什么它的存在依据是 object boundary，而不是 authoring 偏好；
- 为什么 selection-first rewrite 需要独立产品身份。

## Accepted Decision

当前 accepted decision 是：

- `apply_rewrite` 作为 DocuTouch 的独立第三编辑工具存在；
- 它的对象定义为“被 numbered excerpt selection 锁定的既有旧区间的删除或重写”；
- 后续围绕该对象的正式文档、实现与教学面都不应把它降格为 `apply_patch` 或 `apply_splice` 的 mode。

## Accepted Rationale

### Object Boundary

- `apply_patch` 处理的是文本差异；
- `apply_splice` 处理的是既有文本跨度之间的转移关系；
- `apply_rewrite` 处理的是 selection-locked old span 的删除或替换。

三者处理的对象不同，
因此工具身份不应折叠。

### Authoring Model

- `apply_patch` 以 old-side evidence + new-side diff body 为 authoring core；
- `apply_splice` 以 source / target relation 为 authoring core；
- `apply_rewrite` 以 selection-first, then rewrite 为 authoring core。

只要核心 authoring model 不同，
就不应仅因部分底层 substrate 可共享而合并 public surface。

### Failure And Repair Model

- `apply_patch` 的主要 repair 对象是 stale context 或 weak old-side evidence；
- `apply_splice` 的主要 repair 对象是 source / target selection 与 transfer legality；
- `apply_rewrite` 的主要 repair 对象是 selected old span 的 truth validation、overlap legality 与 `WithBlock` structure。

这说明三者并不是同一错误模型下的不同语法糖。

### Governance Consequence

若不先固定 `apply_rewrite` 的独立身份，
后续需求会持续把 selection-led replacement、anchored insertion、patch hunk 变体与 transfer action 混写到一起，
最终破坏高稳定性工具最依赖的窄边界与 canonical surface。

## Rejected Alternatives

当前被拒绝的说法包括：

- 把 `apply_rewrite` 视为 `apply_patch` 的 rewrite mode；
- 把 `apply_rewrite` 视为 `apply_splice` 加一段 authored payload；
- 仅因 token 成本、实现复用或教学便利，就主张并入已有工具；
- 因为 delete 可实现为 replace-to-empty，就否认 delete action 的第一类对象地位。

## Operational Consequence

这项裁决导出的协作关系是：

- 当对象是 diff-style textual revision 时，使用 `apply_patch`；
- 当对象是 existing span transfer 或 anchored insertion 时，使用 `apply_splice`；
- 当对象是 selection-locked old-span replacement or deletion 时，使用 `apply_rewrite`。

这里的分流依据是对象变化，
不是工具竞争。

## Source Basis

- `temporary/apply_rewrite 正式设计思想与详细规格.md`
- {ref}`knowledge-interfaces-apply-patch-semantics`
- {ref}`knowledge-interfaces-apply-splice-spec`
- {ref}`knowledge-interfaces-apply-rewrite-spec`
