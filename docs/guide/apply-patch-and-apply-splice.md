# `apply_patch` And `apply_splice`

## `apply_patch`

`apply_patch` 接受 patch-shaped 输入，用来创建、更新、删除或重命名文件。它沿用 upstream `apply-patch` 的输入形态，但当前 runtime 已经有明确的 DocuTouch 分叉：

- connected file groups 的原子提交
- disjoint groups 的 `PartialSuccess`
- 成功路径上的 warning block
- 便于继续修复的 diagnostics
- 对 `.docutouch/failed-patches/*.patch` repair artifact 的 CLI replay 支持

当前工具文档中的最小 authored shape是：

```text
*** Begin Patch
*** Update File: src/app.py
@@
-print("Hi")
+print("Hello")
*** End Patch
```

如果普通 context 还不够唯一，`apply_patch` 允许额外加一个可选的 numbered anchor，例如 `@@ 120 | def handler():`。

默认 mode 是 `header_only`：只解释 numbered `@@` header。只有在人工通过环境变量或 CLI 显式打开 `full` mode 时，body-level 的 dense numbered old-side evidence 才会进入解释路径。

如果 patch 里只有一组失败，common path 仍然保持紧凑；如果 patch 部分落盘，failure surface 会把 committed changes、failed file groups 和 attempted changes 都保留下来。

适合用 `apply_patch` 的任务包括：

- 重写一段函数实现
- 批量修改多个互不相关的文件
- 创建新文件或重命名文件
- 需要利用 patch diagnostics 继续 repair loop 的修改任务

## `apply_splice`

`apply_splice` 只处理既有文本跨度。它的输入是 line-oriented 的 source/target selection program。它适合搬运、复制、替换或删除已经存在的文本片段。

当前 interface contract 的关键点是：

- 选择器是 line-oriented 的
- 使用 absolute line numbers
- 使用 visible content 做 double-lock validation
- omission marker 是正式语法的一部分
- `Delete Span` 是 first-class action
- 当 source selection 命中 source 文件最后一行且缺少 terminal newline，而当前 target-side boundary 会把两行拼在一起时，runtime 会只在本次结果侧补出 line separator；它不会回写 source 文件本身

典型形态如下：

```text
*** Begin Splice
*** Copy From File: source.py
@@
12 | def build_context(...)
... source lines omitted ...
19 |     return "strict"
*** Append To File: target.py
*** End Splice
```

适合用 `apply_splice` 的任务包括：

- 把一段已存在代码从一个文件搬到另一个文件
- 复制一段已存在实现到目标位置
- 删除一整段既有跨度
- 用一段现有块替换另一段现有块

## 为什么分成两个工具

这里处理的是两类对象：

- `apply_patch` 处理文本差异与新文本状态
- `apply_splice` 处理既有文本片段之间的转移关系

对象不同，工具身份就保持分立。共享底层文件修改基础能力，不推出共享产品身份。
