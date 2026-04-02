(knowledge-interfaces-apply-rewrite-spec)=
# Apply Rewrite Interface Spec

## Role

本页记录 `apply_rewrite` 的 accepted interface contract。

它回答：

- 工具的 public identity 是什么；
- 允许哪些 file operation 与 rewrite action；
- selection 与 `WithBlock` 的稳定 authoring surface 是什么；
- execution、atomicity 与 diagnostics 在接口层面如何被理解。

## Interface Identity

`apply_rewrite` 是面向 selected old-span rewrite 的结构化编辑工具。

它允许调用者表达：

- 创建新文件，并提供完整 authored contents；
- 删除既有文件；
- 重命名文件；
- 在既有文件中选定一个连续旧区间，并将其删除或替换为新文本。

它不允许调用者表达：

- patch hunk 风格的 diff body；
- `Append`、`Insert Before`、`Insert After` 一类 anchored insertion；
- `Copy From File` / `Move From File` 一类 source-to-target transfer；
- fuzzy matching 或自由上下文搜索。

因此：

- `apply_patch` 处理 diff-style textual revision；
- `apply_splice` 处理 existing span transfer；
- `apply_rewrite` 处理 selection-locked old-span deletion or replacement。

## Contract Boundary

`apply_rewrite` 的产品边界刻意保持狭窄。

它不承担：

- `apply_patch` 的 hunk grammar；
- `apply_splice` 的 source / target 双侧 action model；
- insertion family；
- 宽松定位或语义级搜索；
- 在同一个 rewrite action 中处理多个分散旧区间。

超出这些边界的能力不属于当前 interface contract；它们不能在 `apply_rewrite` 名下被静默接纳。

## File Operation Basis

当前完整 file operation basis 为：

- `Add File` + `WithBlock`
- `Delete File`
- `Update File` + optional `Move to` + one or more rewrite actions

其中 `Update File` 下的 rewrite action 只有两种 surface 终止：

- selection + `*** Delete`
- selection + `WithBlock`

接口层面的稳定表达是：

- delete action 本质上是 selected old span 的 remove；
- replace action 本质上是 selected old span 的 replace；
- 二者共享同一个 selection-first object model。

## Canonical Authored Shape

Canonical update shape:

```text
*** Begin Rewrite
*** Update File: src/app.py
@@ rewrite the selected context block to use the new mode contract
120 | def build_context(...)
... lines omitted ...
128 |     return "strict"
*** With
def build_context(...):
    return "flexible"
*** End With
*** End Rewrite
```

Delete shape:

```text
*** Begin Rewrite
*** Update File: src/app.py
@@ remove the obsolete helper after migration
120 | def obsolete_helper(...)
... lines omitted ...
128 |     return old_value
*** Delete
*** End Rewrite
```

Add-file shape:

```text
*** Begin Rewrite
*** Add File: docs/guide.md
*** With
# Guide

Use apply_rewrite for selection-locked replacement.
*** End With
*** End Rewrite
```

Move-and-rewrite shape:

```text
*** Begin Rewrite
*** Update File: src/app.py
*** Move to: src/main.py
@@ rename the file and rewrite the app name constant
5 | APP_NAME = "demo"
*** With
APP_NAME = "production"
*** End With
*** End Rewrite
```

## Selection Contract

`apply_rewrite` 的旧侧 surface 只有 selection block。

selection header 的当前 grammar 形态为：

```text
Selection := "@@" [ " " intent_comment ] NEWLINE SelectionItem+
intent_comment := /(.*)/
```

稳定约束为：

- selection 是 line-oriented；
- selection 使用 absolute 1-indexed line numbers；
- selection 锁定 contiguous old span，而不是 sparse sample；
- 允许 vertical omission；
- 不允许 horizontal truncation；
- omission marker 只允许 `... lines omitted ...`；
- validation 采用 double-lock：line number + visible line content。
- `@@` 允许在同一行后方携带一条可选的单行自然语言意图注释；
- 该注释只能留在 `@@` 同一行，下一行必须进入 numbered selection body；
- 该注释是 parser-supported compatibility surface；
- 该注释服务 authoring 可读性与模型自解释；
- 该注释不替代 numbered selection，也不进入 runtime 执行语义。

当前 accepted authored default 还包括：

- multi-line selection 默认优先采用 boundary anchors + omission marker 的 compact 形态；
- 当 boundary-anchored selection 已足以 truthfully 锁定 intended span 时，不应为了“更完整”而枚举中间 numbered lines；
- 只有当较短写法会引入歧义，或 interior lines 本身就是 intended evidence 时，才展开内部 numbered lines；
- 任一写出的 numbered line 都必须保留 full visible content。

一个 canonical selection 头可以写成：

```text
@@ remove duplicate bootstrap logic after startup centralization
12 | def old_bootstrap(...)
... lines omitted ...
48 |     return bootstrap_state
```

这里 `@@` 后方的单行 comment 只是说明当前 action 的意图；真正参与执行语义的仍然是 numbered selection body。

## `WithBlock` Contract

`WithBlock` 同时服务 `Add File` 与 replace-style rewrite action。

接口层面的稳定约束为：

- `WithBlock` 是 literal text payload；
- payload 行不带 patch 前缀；
- payload 中每一行按原样进入新文本；
- `*** End With` 是必须的显式终止标记；
- payload 可以为空；
- 但 public guidance 不应把 empty `WithBlock` 教成 delete 的 canonical 形态。
- 当 replacement payload 不带 terminal newline 且 rewritten result 在其后仍保留 suffix 时，runtime 必须在结果组合阶段维持 line boundary，防止 payload 末尾与保留 suffix 灾难性拼成一行；
- 当 replacement 直达文件尾且 result 不再保留 suffix 时，runtime 保留 authored EOF 状态，不额外补换行。

## Operational Semantics

接口层面的操作语义如下：

- `Add File`：以 `WithBlock` 的 payload 作为新文件完整内容；
- `Delete File`：删除既有文件；若路径不存在，则按 target-state hard failure 处理；
- `Update File`：在 original snapshot 上解析并验证所有 rewrite actions，再统一 compose 结果；
- `Move to`：在 result content 计算完成后应用文件级 rename。

针对 `Update File`，当前锁定的语义包括：

- 同一 file op 下的所有 selection 都基于同一 original snapshot 解释；
- 前一个 rewrite action 的结果不会改变后一个 action 的 selection 基线；
- 多个 non-overlapping rewrite actions 的 compose 以 original snapshot 上的 range order 稳定执行；
- 同一 `Update File` 内，resolved old spans 若发生 overlap，应返回稳定 overlap-class diagnostic；
- delete action 可在 runtime 内部被实现为 replace-to-empty，但 public identity 仍保持 delete/rewrite 区分。
- `Move to` 与 rewrite 同时成功时，success summary 必须保留最终路径 accounting：`A destination` + `D source`，不压成抽象 `M`。

## Compatibility Notes Boundary

当前 interface contract 接受对 runtime overwrite reality 的披露，
但明确将其归类为 compatibility reality，而不是 canonical 教学面。

这包括：

- `Add File` 命中既有文件时，runtime 可以替换其内容；
- `Move to` 命中既有 destination 时，runtime 可以替换目标内容；
- 这些行为若发生，success surface 可伴随 warning 或等价兼容性披露；但它们不是 primary authoring advice。

因此：

- 可以披露 runtime reality；
- 不能把 overwrite-via-Add 或 overwrite-via-Move 教成 preferred tactic。

## Success And Failure Contract

success surface 继承 DocuTouch 当前工具家族的紧凑结果摘要：

```text
Success. Updated the following files:
A path/to/file
M path/to/other
D obsolete.txt
```

其中 move-and-rewrite 的 accepted summary shape 为：

```text
Success. Updated the following files:
A path/to/destination
D path/to/source
```

接口层面可依赖的性质包括：

- `A/M/D` 是 affected-path summary，而不是逐动作回放；
- final-path accounting 优先于抽象动作压缩，因此 move-and-rewrite 不被简写成单行 `M`；
- connected file groups 不发生半提交；
- independent file groups 允许 partial success；
- partial success 必须保留 committed accounting 与 failed-group accounting；
- diagnostics 应尽量指向最真实的 authored blame location；
- selection mismatch 不允许退回 fuzzy fallback。

建议 diagnostic family 包括：

- `REWRITE_PROGRAM_INVALID`
- `REWRITE_SELECTION_INVALID`
- `REWRITE_SELECTION_TRUNCATED`
- `REWRITE_SELECTION_OVERLAP`
- `REWRITE_TARGET_STATE_INVALID`
- `REWRITE_WRITE_ERROR`
- `REWRITE_PARTIAL_UNIT_FAILURE`

其中 `REWRITE_TARGET_STATE_INVALID` 必须覆盖 `Delete File` 命中不存在路径的硬失败场景；
repair guidance 应明确要求重新读取 workspace，并在目标已不存在时去掉对应 delete。

## Acceptance Boundary

本页只记录 `apply_rewrite` 当前已接纳的 interface semantics。

它不承担：

- implementation schedule；
- parser / runtime 模块拆分方案；
- work package sequencing；
- 其他 action family 的扩张讨论。
