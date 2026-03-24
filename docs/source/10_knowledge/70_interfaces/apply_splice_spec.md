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

## Operational Semantics

接口层面的操作语义如下：

- `Copy + Append`：读取 source range，追加到目标文件末尾，source 保持不变。
- `Move + Append`：读取 source range，追加到目标文件末尾，并删除 source range。
- `Copy / Move + Insert Before / After`：以 target anchor selection 为插入定位，仅在 `Move` 时删除 source。
- `Copy / Move + Replace`：以 source block 替换 target range，仅在 `Move` 时删除 source。
- `Delete Span`：删除 source range，不进行目标侧写入。

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
- same-file overlap 对 `Insert Before`、`Insert After`、`Replace` 属于非法情况，应返回稳定 overlap-class diagnostic；
- `Append To File` 可以创建缺失目标文件；
- `Insert Before In File`、`Insert After In File`、`Replace In File` 要求目标文件与目标 selection 已存在；
- source text verbatim preserved，包括 newline bytes。

## Acceptance Boundary

本页只记录 `apply_splice` 当前已接纳的 interface semantics。

它不承担：

- implementation schedule；
- extraction sequencing；
- architecture reuse rationale；
- future macro-operations debate。
