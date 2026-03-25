(knowledge-interfaces-apply-splice-spec)=
# Apply Splice Interface Spec

## Role

本页记录 `apply_splice` 的 accepted interface contract。

它回答：

- 工具的 public identity 是什么；
- 允许哪些 action；
- 选择器与 authored surface 采用什么稳定写法；
- execution、atomicity 与 diagnostics 在接口层面如何被理解。

## Interface Identity

`apply_splice` 是面向既有文本跨度的结构化转移工具。

它允许调用者表达：

- 选择哪一段已存在文本；
- 这段文本是 `Copy`、`Move` 还是 `Delete Span`；
- 若发生转移，目标侧采用 `Append`、`Insert Before`、`Insert After` 或 `Replace`。

它不允许在 splice 程序内部内联创作新文本。

因此：

- `apply_patch` 用于 author / rewrite text；
- `apply_splice` 用于 relocate / duplicate / delete existing text spans。

## Contract Boundary

`apply_splice` 的产品边界刻意保持狭窄。

它不承担：

- free-form patch language；
- selected span 的局部改写；
- JSON 坐标载荷式主 authored surface；
- 泛化 refactoring engine；
- fuzzy 或 semantic matching。

若未来需要这些能力，应视为新的产品工作，而不是在当前接口下静默膨胀。

## Action Basis

当前完整 action basis 为九个原语：

- `Delete Span From File`
- `Copy From File` + `Append To File`
- `Move From File` + `Append To File`
- `Copy From File` + `Insert Before In File`
- `Copy From File` + `Insert After In File`
- `Move From File` + `Insert Before In File`
- `Move From File` + `Insert After In File`
- `Copy From File` + `Replace In File`
- `Move From File` + `Replace In File`

其中：

- transfer family 由 `Copy/Move × Append/Insert Before/Insert After/Replace` 构成；
- removal family 目前仅包含 `Delete Span`。

当前 accepted authored default 还包括：

- 当调用者要删除一段既有 contiguous span 且不发生 target-side transfer 时，默认优先使用 `Delete Span`；
- 尤其在 removal body 较大时，不应为了沿用 patch 心智而把既有删除任务改写成需要重述整段 removed text 的 authored surface。

## Canonical Authored Shape

接口采用 envelope-shaped、patch-like authored surface，但不复用 patch hunk grammar。

Canonical shape:

```text
*** Begin Splice
*** Copy From File: source.py
@@
120 | def build_context(...)
... source lines omitted ...
128 |     "mode": "strict",
*** Append To File: target.py
*** End Splice
```

Replace shape:

```text
*** Begin Splice
*** Move From File: source.py
@@
120 | def build_context(...)
... source lines omitted ...
128 |     "mode": "strict",
*** Replace In File: target.py
@@
45 | old block start
... target lines omitted ...
52 | old block end
*** End Splice
```

Delete shape:

```text
*** Begin Splice
*** Delete Span From File: source.py
@@
120 | def obsolete_helper(...)
... source lines omitted ...
128 |     return old_value
*** End Splice
```

## Selection Contract

source / target selection 必须保持低歧义。

稳定约束为：

- selection 是 line-oriented；
- selection 携带 absolute 1-indexed line numbers；
- 允许 vertical omission；
- 不允许 horizontal truncation；
- 采用 double-lock validation。

double-lock validation 的两把锁是：

- absolute line numbers 锁定 intended span；
- displayed line content 锁定边界行的 semantic identity。

在 selection 内部，authoritative omission markers 为：

- source 侧：`... source lines omitted ...`
- target 侧：`... target lines omitted ...`

这些 omission marker 表示 contiguous range 的压缩写法，而不是 sparse sample。

当前 accepted authored default 还包括：

- multi-line selection 默认优先采用 boundary anchors + omission marker 的 compact 形态；
- 当省略后的 boundary-anchored selection 已足以 truthfully 锁定 intended span 时，不应为“更完整”而枚举中间 numbered lines；
- 只有当较短写法会产生歧义，或中间全文本本身就是 intended evidence 时，才展开 interior numbered lines；
- 任一被写出的 numbered line 仍必须保留 full visible content，而不是局部片段。

## Operational Semantics

接口层面的操作语义如下：

- `Copy + Append`：读取 source range，追加到目标文件末尾，source 保持不变。
- `Move + Append`：读取 source range，追加到目标文件末尾，并删除 source range。
- `Copy / Move + Insert Before / After`：以 target anchor selection 为插入定位，仅在 `Move` 时删除 source。
- `Copy / Move + Replace`：以 source block 替换 target range，仅在 `Move` 时删除 source。
- `Delete Span`：删除 source range，不进行目标侧写入。

对于 transfer family，还存在一条 targeted newline-boundary rule：

- 若 source selection 命中 source 文件的最后一行且该行不带 terminal newline；
- 并且当前 target-side boundary 若按 raw byte transfer 继续执行会把两段本应分属不同行的文本拼接到同一行；
- runtime 应在 result-side compose 阶段补入 target-style line separator，以维持 line-oriented transfer 结果；
- 该补入仅作用于本次 target/result 组合，不回写 source 文件本身。

同一 envelope 内可包含多个显式 splice action；每个 action 仍然是独立的 source-to-target instruction。

## Atomicity And Diagnostics

`apply_splice` 继承 DocuTouch 对 connected mutation unit 的严肃性。

接口层面要求：

- 每个 splice action 是一个 connected mutation unit；
- `Move` 不得留下“目标已写入、源未移除”或其反向的半完成状态；
- 多文件更新必须遵守 connected update 的 atomicity discipline。

diagnostics contract 保持与现有 patch tooling 风格一致：

- stable codes；
- compact, repair-oriented output；
- truthful blame locations；
- preserved repair accounting when partial success exists。

## Same-File And Destination Rules

当前锁定的 interface rules 包括：

- same-file source 与 target selection 均针对 original snapshot 解析；
- same-file anchored target action 在 source 与 target ranges 针对该 original snapshot 保持 non-overlapping 时是允许的；
- 对 `Insert Before`、`Insert After`、`Replace` 而言，same-file overlap 仅在 source range 与 anchored target range 重叠时才属于非法情况，并应返回稳定 overlap-class diagnostic；
- `Append To File` 可以创建缺失目标文件；
- `Insert Before In File`、`Insert After In File`、`Replace In File` 要求目标文件与目标 selection 已存在；
- ordinary transfer 继续保留 source text 与 newline bytes 的主要可见内容；
- 但若 source selection 命中 EOF final line 且缺少 terminal newline，当前 target-side boundary 又会导致 same-line concatenation，runtime 应优先维持 line boundary，而不是把 EOF-without-newline state 原样扩散到目标结果中；
- 这类 newline-boundary normalization 只影响本次 target/result 组合，不改写 source 文件自身的 EOF 状态。

## Acceptance Boundary

本页只记录 `apply_splice` 当前已接纳的 interface semantics。

它不承担：

- implementation schedule；
- extraction sequencing；
- architecture reuse rationale；
- future macro-operations debate。
