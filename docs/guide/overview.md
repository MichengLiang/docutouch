# Overview

DocuTouch 是一组面向 LLM coding agents 的结构化文件工具。

当前仓库里最重要的两个工具是：

- `apply_patch`
- `apply_splice`

`apply_patch` 负责 patch-shaped 结构化写入。它沿用 upstream `apply-patch` 的输入形态，同时把 partial success、connected file groups、warning block、便于继续修复的 diagnostics 和 file-backed repair loop 带进当前 runtime。普通 context 不够唯一时，还可以可选地使用 `@@ N | visible text` 作为 numbered assist；默认 mode 只解释这种 numbered header。

`apply_splice` 负责既有文本跨度的复制、移动、删除和替换。它独立处理既有片段之间的结构关系，不 author 新文本。

配套工具围绕同一条工作流服务：

1. 定位工作区
2. 列出目录
3. 读取上下文
4. 搜索相关文本
5. 应用结构化修改
6. 接收可诊断反馈

DocuTouch 主要通过 MCP / injection 这类代理工具接入方式使用。裸 `docutouch` 命令直接进入 stdio MCP server；CLI 继续保留，并且通过 `docutouch patch`、`docutouch read` 这类顶层子命令为 repair loop 提供 stdin/file 两种进入方式。

如果你想先看一条可执行的启动路径，继续阅读 [quickstart.md](quickstart.md)。如果你想先看所有公开工具，继续阅读 [tool-surfaces.md](tool-surfaces.md)。
