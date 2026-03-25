# DocuTouch

语言版本：

- [English](README.md)
- [简体中文](README.zh-CN.md)

DocuTouch 是一组面向 LLM coding agents 的结构化文件工具。

这个仓库围绕两个结构化编辑工具展开：

- `apply_patch`
- `apply_splice`

前者沿用 OpenAI Codex `apply-patch` 的输入形态，同时把执行行为打磨成更适合代理工作流的样子；后者专门处理既有文本跨度的复制、移动、删除与替换。

配套工具包括：

- `list_directory`
- `read_file`
- `search_text`

## 什么时候会用到它

DocuTouch 适合下面这类工作：

- 需要把一次文件修改写成结构化 patch，而不是自然语言编辑指令
- 一个大 patch 里有多组相互独立的修改，希望正确的部分先落盘，失败的部分单独返修
- 需要把失败 patch 直接变成一条可重放的 repair loop
- 需要搬运、重排、复用已经存在的代码或文本片段，不想把整段内容再 author 一遍

## `apply_patch`

`apply_patch` 是 patch-shaped 结构化写入工具。它仍然使用熟悉的 authored shape：

```text
*** Begin Patch
*** Update File: src/app.py
@@
-print("Hi")
+print("Hello")
*** End Patch
```

当前工作区里的 runtime 在 upstream `apply-patch` baseline 之上增加了下面这些对象：

- connected file groups 的原子提交
- disjoint file groups 的 `PartialSuccess`
- 更利于继续修复的 diagnostics
- 成功路径上的独立 warning block
- patch 失败后的 file-backed repair loop

这意味着一个大 patch 不必在“有一处失败”时整体作废。已经正确且彼此独立的部分可以先落盘，失败的部分会以 committed changes / failed file groups / attempted changes 的方式被单独列出。

warning 采用独立 code-bearing block。例如：

```text
Success. Updated the following files:
M notes.md

warning[ADD_REPLACED_EXISTING_FILE]: Add File targeted an existing file and replaced its contents
  --> notes.md
  = help: prefer Update File when editing an existing file
```

partial failure 走的是 repair-first surface。headline 以稳定 error code 开始，例如：

```text
error[PARTIAL_UNIT_FAILURE]: patch partially applied
```

随后继续给出：

- `committed changes:`
- `failed file groups:`
- 每个 `failed_group[n]`
- `attempted changes:`
- `help:`

这条设计的直接价值是：

- 少重试已经成功的部分
- 少重发已经落盘的 patch 片段
- 少浪费 token 在重复上下文上
- 把下一轮修复压缩成失败的那一小段

CLI 也围绕 repair loop 打磨过。`patch` 既支持 stdin，也支持 patch file。对于 `.docutouch/failed-patches/*.patch` 这样的 repair artifact file，CLI 会恢复其所属 workspace anchor，使“改失败 patch 文件再重放”成为一条直接路径。

更细的 lineage 与分叉说明见：

- [codex-apply-patch/README.md](codex-apply-patch/README.md)
- [codex-apply-patch/UPSTREAM_LINEAGE.md](codex-apply-patch/UPSTREAM_LINEAGE.md)
- [codex-apply-patch/LOCAL_DIVERGENCES.md](codex-apply-patch/LOCAL_DIVERGENCES.md)

## `apply_splice`

`apply_splice` 是一个单独的工具，处理对象与 `apply_patch` 不同。

- `apply_patch` 处理文本差异与新文本状态
- `apply_splice` 处理既有文本片段之间的转移关系

它的 authored surface 直接声明：

- 从哪里选一段已存在文本
- 这段文本是 `Copy`、`Move` 还是 `Delete Span`
- 目标侧是 `Append`、`Insert Before`、`Insert After` 还是 `Replace`

最小形态像这样：

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

它的关键点在对象边界：

- 选择器是 line-bearing 的
- 采用 absolute line numbers + visible content 的 double-lock validation
- omission marker 是一等对象
- `Delete Span` 是 first-class action
- connected mutation unit 同样保持原子性

这类表述对大模型友好，因为它避免为了做一次已有文本搬运而重述整段正文。对象已经存在，就直接以选择器和关系来表达。

## 怎么选工具

如果你的任务是“把某段文本改成另一段文本”，使用 `apply_patch`。

例如：

- 修改函数体
- 新增文件
- 删除文件
- 重命名文件并顺便改内容

如果你的任务是“把已经存在的一段文本复制、移动、删除或替换到别的位置”，使用 `apply_splice`。

例如：

- 把一个 helper 从一个文件搬到另一个文件
- 复制一个已有配置块到目标文件
- 删除一整段已存在跨度
- 用一段现有实现替换另一段现有实现

## 安装与前提

当前公开安装路径是 source build。

前提：

- Rust toolchain
- Cargo
- 如果要使用 `search_text`，需要 `rg`（ripgrep）在 PATH 中可用

构建当前工作区：

```bash
cargo build
```

## 快速开始

启动 stdio MCP server：

```bash
cargo run -p docutouch-server
```

也可以显式写成：

```bash
cargo run -p docutouch-server -- serve
```

如果要直接从 CLI 调用：

```bash
cargo run -p docutouch-server -- list docutouch-server/src
cargo run -p docutouch-server -- read README.md --line-range 1:40
cargo run -p docutouch-server -- search apply_patch docutouch-server/src --view full
```

## MCP 配置示例

下面是一个最小的 stdio MCP server 配置示例：

```json
{
  "command": "cargo",
  "args": ["run", "-q", "-p", "docutouch-server"],
  "env": {
    "DOCUTOUCH_DEFAULT_WORKSPACE": "/absolute/path/to/project"
  }
}
```

如果宿主会在连接后立刻调用 `set_workspace`，`DOCUTOUCH_DEFAULT_WORKSPACE` 可以省略。

## CLI repair loop

`patch` 支持 stdin，也支持 patch file。失败后的典型 repair loop 是：

```bash
cargo run -p docutouch-server -- patch .docutouch/failed-patches/1712345678901-0.patch
```

或者在编辑 patch 文本后从 stdin 重放：

```bash
cat retry.patch | cargo run -p docutouch-server -- patch
```

对 `.docutouch/failed-patches/*.patch` 这类 repair artifact file，CLI 会恢复它所属的 workspace anchor。

## 配套工具

- `list_directory`
  以 ASCII 树建立文件清单。

- `read_file`
  以单文件为主路径读取上下文，支持行号、分段和 sampled inspection。

- `search_text`
  用 ripgrep 做底层搜索，并把结果按文件分组返回。

## 文档入口

- [docs/README.md](docs/README.md)
  公开文档封面。

- [docs/guide/README.md](docs/guide/README.md)
  guide 入口。

- [docutouch-server/README.md](docutouch-server/README.md)
  `docutouch-server` 模块入口。

- [docutouch-core/README.md](docutouch-core/README.md)
  `docutouch-core` 模块入口。

- [codex-apply-patch/README.md](codex-apply-patch/README.md)
  `codex-apply-patch` 模块入口。

## 项目状态

当前主工具面已经可用，对外文档与公开表面仍在继续收口。

## Contributing

- [CONTRIBUTING.md](CONTRIBUTING.md)
  贡献说明与文档约定。

## 测试

运行 `docutouch-core` 与 `docutouch-server` 的回归测试：

```bash
cargo test -p docutouch-core -p docutouch-server
```

运行 `codex-apply-patch` 的回归测试：

```bash
cd codex-apply-patch
cargo test
```

## License

Apache-2.0.
