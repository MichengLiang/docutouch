# Tool Surfaces

## `apply_patch`

`apply_patch` 是当前工作区里的核心结构化写入工具。它接受 patch-shaped 输入，runtime 会把相关文件操作按 connected file groups 组织，并在彼此独立的 groups 之间允许 `PartialSuccess`。输出保留 `A/M/D` summary、warning block、committed changes、failed file groups 与 attempted changes，目标是缩短失败后的 repair loop。需要消歧时可以可选地使用 `@@ N | visible text`；默认 mode 只解释 numbered header，不默认上浮 body-level dense numbering。

## `apply_splice`

`apply_splice` 是面向既有文本跨度的结构化转移工具。它用 source/target selection、Copy/Move/Delete Span 与 Append/Insert/Replace 去表达片段关系。这使它更适合重排、复用和搬运已经存在的代码或文本。

## `list_directory`

列出目录内容，并以 ASCII 树形式呈现。这个工具适合建立文件清单、判断阅读范围，以及为下一步 `read_file` 或 `search_text` 缩小范围。

## `read_file`

读取单个文件。它支持行号、行范围、sampled inspection 和横向裁切。除了普通的 relative/absolute path 之外，`read_file.relative_path` 也接受 `pueue-log:<id>` task-log handle，可直接读取 `wait_pueue` 返回的日志句柄。DocuTouch 当前不把多文件正文拼成一个巨型返回体，主路径是保持文件边界稳定。

## `search_text`

使用 ripgrep 做底层搜索，同时把结果按文件分组返回。这让“先搜索、再按文件读取”的工作流更自然，也更适合模型继续跟踪上下文。`search_text.path` / `path[]` 除了文件、目录 path 之外，也接受 `pueue-log:<id>` task-log handle，可直接搜索 `wait_pueue` 返回的日志句柄。

## `wait_pueue`

等待一个或多个 Pueue 后台 task 到达终态，并返回稳定的 wait summary surface。终态 task block 会附带 `pueue-log:<id>` 格式的 `log_handle`，供后续 `read_file` 或 `search_text` 继续检查日志。
