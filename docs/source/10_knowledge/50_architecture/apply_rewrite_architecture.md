(knowledge-architecture-apply-rewrite-architecture)=
# Apply Rewrite Architecture

## Role

本页承载 `apply_rewrite` 的 accepted architecture。

它回答的是：

- `apply_rewrite` 在当前产品体系中的结构位置是什么；
- 哪些 parser / selection / runtime / diagnostics / transport layers 必须保持 rewrite-owned；
- 哪些 correctness substrate 可以共享；
- 为什么 shared substrate 不等于 patch-owned 或 splice-owned semantic layer。

## Accepted Boundary

本页处理的是 `apply_rewrite` 的 accepted solution structure，而不是：

- implementation schedule；
- work package sequencing；
- release readiness 结果；
- temporary design notes。

因此，阶段状态与推进计划不应写入本页。

## Architectural Thesis

`apply_rewrite` 应作为 DocuTouch-owned tool stack 落地，
而不是并入 `apply_patch` 或 `apply_splice` 的 semantic layer。

accepted architecture direction 是：

- 维持 `apply_rewrite` 的独立工具身份；
- 仅共享最低必要的 correctness substrate；
- 在 DocuTouch-owned layer 中建立独立的 rewrite parser、selection resolution、runtime compose、presentation 与 transport wiring。

## System Placement

当前 architecture layers 的职责应区分为：

- `codex-apply-patch`：现有 patch baseline 与可被抽取的 lower correctness substrate；
- `docutouch-core`：shared filesystem/runtime substrate 与 DocuTouch-owned semantic layers 的宿主；
- `docutouch-server`：MCP / CLI transport registration、workspace negotiation 与 outer-surface glue；
- `apply_rewrite`：建立在 shared substrate 之上的独立工具层，而不是 patch mode 或 splice mode。

## Shared Versus Owned Boundary

### May Be Shared

下列能力在保持 rewrite-agnostic 的前提下可被 shared substrate 承载：

- path identity and normalization
- connected mutation unit grouping
- staged commit / rollback machinery
- affected-path accounting and `A/M/D` summarization
- generic diagnostic rendering helpers
- generic path display helpers
- numbered-excerpt selection resolution substrate 的通用部分，但前提是它不携带 patch hunk 或 splice source/target 语义

### Must Remain Rewrite-Owned

下列能力必须保持为 `apply_rewrite` own layer：

- public tool identity and user-facing boundary
- rewrite envelope grammar and parser
- `*** With` / `*** End With` block structure
- selection-led rewrite action model
- overlap legality for multiple rewrite actions inside one `Update File`
- original-snapshot compose rule for all selections in the same file op
- rewrite-specific diagnostics vocabulary and blame hierarchy
- prompt-facing tool docs that teach `apply_rewrite` itself
- transport wiring and tool registration for `apply_rewrite`

## Core Architecture Commitments

### 1. Selection-First Runtime

`apply_rewrite` 的 runtime 应先解析并 resolve 同一 `Update File` 内的全部 selections，
再统一 compose result content。

不得采用“先改前一个 action，再按修改后文件解释后一个 selection”的串行快照模型。

### 2. Rewrite Actions Stay Single-Purpose

每个 rewrite action 只表达一件事：

- delete selected old span；或
- replace selected old span with authored payload。

architecture 不应为方便扩展而把 `append`、`insert before`、`insert after` 等非本体动作预埋为当前 contract 的隐含分支。

### 2.5 `@@` Same-Line Intent Comment Stays Explanatory

`@@` 在 `apply_rewrite` 中继续承担 selection block 的开始标记。

accepted architecture 允许 `@@` 在同一行后方携带一条可选单行自然语言意图注释，
例如：

```text
@@ remove duplicate bootstrap logic
```

但这条注释的地位必须保持为 explanatory compatibility surface：

- parser 可以接受并保留；
- runtime 不以其驱动 selection resolution；
- runtime 不以其驱动 replacement validation、overlap legality 或 success/failure accounting；
- 下一行仍必须进入 numbered selection body。

### 3. `WithBlock` Is A Real Structural Boundary

`*** With` / `*** End With` 不只是 parser 细节，
而是 rewrite payload 与后续 action / file op header 的稳定边界。

accepted architecture 应保留这一显式终止结构，
而不是把“遇到下一个 `***` 就结束 body”作为真实 contract。

另外，runtime compose layer 必须承担 result-side line-boundary normalization：

- 当 replacement payload 无 terminal newline 且 compose 后仍有保留 suffix 时，结果组合必须维持行边界，禁止把 payload 尾部与 suffix 首行拼成同一行；
- 当 replacement 直达 EOF 且无 suffix 时，compose 保留 authored EOF 状态。

### 4. Compatibility Behavior Stays Secondary

若 runtime 允许：

- `Add File` 覆盖既有文件；
- `Move to` 覆盖既有 destination；

这些属于 compatibility behavior。
architecture 可以复用现有 shared commit substrate 实现这些 reality，
但不应反向塑造 prompt-facing primary semantics。

当这些 overwrite reality 实际发生时，rewrite-owned presentation 可以在 success summary 之外追加 warning；
这属于 compatibility disclosure，不是 canonical authoring signal。

### 5. Diagnostics Family Is Rewrite-Owned

`apply_rewrite` diagnostics 应与现有 DocuTouch 风格同族，
但不能因为 patch 或 splice 已有 diagnostics，
就默认 rewrite 可以直接继承其 code family 或 blame hierarchy。

其中 `Delete File` 命中不存在路径必须被视为 target-state hard failure，
而不是 silent success 或 warning-only outcome。

### 6. Success Presentation Must Stay Final-Path-Truthful

presentation layer 必须按最终路径结果汇报 success accounting。

因此 move-and-rewrite 的成功面固定为：

- `A destination`
- `D source`

不得为了追求抽象统一而把这类结果压缩成单行 `M`。

## Internal Structure View

接受的内部结构可概括为：

1. parse rewrite program
2. resolve file paths and file-operation grouping
3. for each `Update File`:
4. load original snapshot
5. resolve all rewrite selections against that snapshot
6. validate non-overlap and action legality
7. build replacement plan ordered by source ranges
8. compose result content
9. apply optional move
10. commit through shared atomic filesystem substrate
11. render success / warning / failure output through rewrite-owned presentation

该结构的关键是：

- parse 与 runtime 分层明确；
- selection resolution 与 compose 分层明确；
- transport 位于 semantic closure 之后。

## Downstream Implications

本页作为 accepted architecture object，可被以下对象引用：

- `70_interfaces/`：说明 public interface contract 与 tool identity
- `60_decisions/`：记录为什么必须独立为第三工具
- `80_operations/`：说明 shared substrate posture 与维护纪律
- `15_process_assets/`：说明 implementation sequencing 与 readiness gates

## Non-Goals

- 不把 `apply_rewrite` 写成 `apply_patch` 的 numbered hunk mode
- 不把 `apply_rewrite` 写成 `apply_splice` 的 authored-text extension
- 不把 insertion family 预先写进当前 architecture truth
- 不以 transport convenience 反向定义 semantic layer
