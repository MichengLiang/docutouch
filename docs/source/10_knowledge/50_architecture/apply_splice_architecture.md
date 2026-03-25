(knowledge-architecture-apply-splice-architecture)=
# Apply Splice Architecture

## Role

本页承载 `apply_splice` 的 accepted architecture。

它回答的是：

- `apply_splice` 在当前产品体系中的结构位置是什么；
- 哪些 correctness substrate 可以共享；
- 哪些 parser / selection / runtime / diagnostics / transport layers 必须保持 splice-owned；
- vendored `codex-apply-patch` 与 `apply_splice` 的关系应如何被约束。

## Accepted Boundary

本页处理的是 `apply_splice` 的 accepted solution structure，而不是：

- implementation schedule；
- work package sequencing；
- temporary closure notes；
- release readiness result。

因此，阶段状态、closeout 与 review log 不应写进本页。

## Architectural Thesis

`apply_splice` 应作为 DocuTouch-owned tool stack 落地，
而不是并入 vendored `codex-apply-patch` grammar/runtime。

accepted architecture direction 是：

- 维持 `apply_patch` 与 `apply_splice` 的产品身份分立；
- 仅共享最低必要的 correctness substrate；
- 在 DocuTouch-owned layer 中建立独立的 splice parser、selection resolver、runtime、presentation 与 transport wiring。

## System Placement

当前 architecture layers 的职责应区分为：

- `codex-apply-patch`：当前 patch-owned baseline 与可被抽取的 lower correctness substrate；
- `docutouch-core`：shared filesystem/runtime shaping substrate 与 DocuTouch-owned semantic layers 的宿主；
- `docutouch-server`：MCP / CLI transport registration、workspace negotiation 与 outer-surface glue；
- `apply_splice`：建立在 shared substrate 之上的独立工具层，而不是 patch mode。

## Shared Versus Owned Boundary

### May Be Shared

下列能力在保持 splice-agnostic 的前提下可被 shared substrate 承载：

- path identity and alias-aware normalization
- connected mutation unit grouping
- staged commit / rollback machinery
- affected-path accounting and `A/M/D`-style summarization
- generic path display helpers
- generic diagnostic rendering helpers
- tiny numbered-excerpt codec helpers, 但前提是它们不携带 source-vs-target 语义

### Must Remain Splice-Owned

下列能力必须保持为 `apply_splice` own layer：

- public tool identity and user-facing boundary
- splice envelope grammar and parser
- source / target selection resolver
- line-bearing selection surface, including omission-backed boundary anchors as the default authored compression shape
- same-file original-snapshot rule and overlap legality
- transfer semantics, including newline / EOF policy and target/result-side line-boundary normalization
- splice-specific diagnostics vocabulary and blame hierarchy
- transfer / removal action semantics
- transport wiring and tool docs that describe `apply_splice` itself

## Internal Substrate Posture

vendored `codex-apply-patch` 在当前仓库中应被视为内部 correctness substrate，
而不是 `apply_splice` 的上游 architecture authority。

这条 accepted posture 的含义是：

- upstream materials 仍可作为 patch baseline 与 divergence disclosure 的 comparison source；
- 但 downstream tool architecture 是否健康，应首先按 DocuTouch 自身的 correctness、maintainability 与 product-boundary judgment 来决定；
- shared substrate extraction 不以“尽量别动 vendored fork”作为默认优先级；
- 只要共享层保持 splice-agnostic，就允许围绕更健康的内部结构进行重构。

换言之：

- `shared` 不等于 `upstream-constrained`
- `vendored` 不等于 `hands-off`

## Core Architecture Commitments

### 1. Shared-First, Not Patch-First

若某项能力未来同时服务 `apply_patch` 与 `apply_splice`，
应优先抽成 shared substrate，
而不是把 `apply_splice` 长期寄生在 patch-owned semantic layer 内。

### 2. Semantic Layers Stay Explicit

`apply_splice` 至少要显式拥有以下模块边界：

- parser
- selection
- runtime
- presentation
- transport/tool wiring

不得把这些语义层混写成一个 transport-heavy adapter。

### 3. Same-File Semantics Are Tool Semantics

same-file snapshot、overlap legality、move translation
属于 splice semantic contract，
不得下沉到 generic shared helper 后再失去工具边界可见性。

### 4. Authored Selections Stay Line-Bearing

`apply_splice` 的 authored selection surface 不应退化为纯坐标载荷。

accepted architecture 允许 runtime 使用 byte offsets、resolved ranges 与其他内部表示，
但 public authored form 仍应保留：

- absolute line numbers；
- visible boundary-line content；
- omission-backed compact compression for contiguous interior lines。

这条约束的职责是让 selection 继续同时承担 coordinate lock 与 local semantic grounding，
而不是把 public surface 缩减成只对 runtime 友好的 opaque coordinate blob。

同样地，source-only removal 也不应为了迁就 patch-shaped authoring 心智而退化回“重述被删除全文”的 surface。

在 `apply_splice` 的 accepted architecture 下：

- `Delete Span` 不是 transfer family 的附属例外，而是既有跨度删除的 first-class authored primitive；
- 对较大的既有删除任务，selection-bearing removal surface 比 restated negative-body patch 更符合 splice 的 product identity。

### 5. Diagnostics Family Is Not Borrowed By Implication

`apply_splice` diagnostics 应与现有 DocuTouch 风格同族，
但不能因为 patch diagnostics 已存在，
就默认 splice 可以直接复用 patch 的 code family、blame hierarchy 或 wording。

### 6. Transport Parity Is Downstream Of Semantic Closure

CLI / MCP wiring 必须建立在 splice semantic layer 之后，
而不是反过来由 transport convenience 决定内层结构。

## Downstream Implications

本页作为 accepted architecture object，可被以下对象引用：

- `70_interfaces/`：说明 public interface contract 与 tool identity
- `80_operations/`：说明内部 substrate posture 与维护纪律
- `15_process_assets/`：说明 implementation sequencing 与 readiness gates
- `60_decisions/`：记录局部 accepted rationale

## Non-Goals

- 不把 `apply_splice` 写成 `apply_patch` 的 mode
- 不把 temporary closure notes 直接当作长期 architecture host
- 不把 upstream closeness 当成 downstream architecture 的首要目标
- 不以尚未完成的 transport wiring 反向定义 semantic truth
