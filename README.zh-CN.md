# DocuTouch

[![release](https://github.com/MichengLiang/docutouch/actions/workflows/release.yml/badge.svg)](https://github.com/MichengLiang/docutouch/actions/workflows/release.yml)
[![npm publish](https://github.com/MichengLiang/docutouch/actions/workflows/npm-publish.yml/badge.svg)](https://github.com/MichengLiang/docutouch/actions/workflows/npm-publish.yml)
[![GitHub release](https://img.shields.io/github/v/release/MichengLiang/docutouch?sort=semver)](https://github.com/MichengLiang/docutouch/releases)
[![npm](https://img.shields.io/npm/v/docutouch)](https://www.npmjs.com/package/docutouch)
[![license](https://img.shields.io/github/license/MichengLiang/docutouch)](LICENSE)

语言版本：

- [English](README.md)
- [简体中文](README.zh-CN.md)

DocuTouch 是一个面向 coding agent 工作流的结构化文件工具工作区。它提供 stdio MCP server、本地 CLI 和 Rust core，用 authored program surface 来读文件、搜文件、检查结构、修改文件，而不是把文件操作交给自然语言编辑请求。

这个项目围绕一个很实际的判断展开：当模型修改文件时，操作应该保留稳定证据、文件边界、修复账目和可重放输入。因此 DocuTouch 把文件上下文、选择器、patch group、AST match、后台日志都当成明确对象来处理。

## 当前交付物

| Surface | 含义 | 主要使用者 |
| --- | --- | --- |
| `docutouch` binary | 来自 `docutouch-server` 的 stdio MCP server 与本地 CLI | Agent、本地操作者、MCP 宿主 |
| `docutouch-core` | 共享 Rust library，承载文件系统工具、搜索 surface、rewrite/splice runtime、patch 呈现与结构搜索 | 维护者、下游 Rust 集成 |
| `codex-apply-patch` | OpenAI Codex `apply-patch` 的 vendored fork，带 DocuTouch runtime 语义 | Patch runtime 与 lineage 审计 |
| `npm/` package | 下载匹配 GitHub Release binary 的薄 Node launcher | 希望使用 `npx docutouch` 或全局 npm 安装的用户 |
| `.github/workflows/release.yml` | 基于 `v*` tag 的 GitHub Release 构建，产出 Windows x64 与 Linux x64 binary | Release 维护者 |
| `.github/workflows/npm-publish.yml` | 带 provenance 的 npm trusted-publishing workflow | Release 维护者 |

DocuTouch 不是 Git、ripgrep、ast-grep 或 Pueue 的第二套实现。它在合适的位置包装这些工具，并把输出整理成更适合 coding agent 消费和重试的形态。

## 什么时候使用

当 coding agent 需要下面这些能力时，可以使用 DocuTouch：

- 在大量阅读前先建立文件清单
- 用稳定行范围和可选行号读取单个文件或单个日志 handle
- 用 ripgrep 搜索代码，同时保留按文件分组的结果面
- 通过 `ast-grep` 做 AST-aware 搜索，并在后续步骤重新打开同一个 match
- 应用结构化 patch，并让互相独立且成功的 file group 先落盘
- 用行号证据锁定旧文本跨度，再替换或删除它
- 复制、移动、删除、替换已经存在的文本跨度，而不是重新 author 整段正文
- 等待 Pueue task，并把返回的日志 handle 继续交给 `read_file` 或 `search_text`

DocuTouch 尤其适合 repair loop。一次失败的操作应该留下足够的结构化证据，让下一次尝试只处理失败部分。

## 工具总览

| MCP tool | 对象边界 | 说明 |
| --- | --- | --- |
| `set_workspace` | relative path 的默认 workspace | 为当前 MCP service instance 保存一个 canonical base path。 |
| `list_directory` | 目录树 | 渲染 ASCII tree，支持 hidden file、gitignored file、timestamp 和 ripgrep/ignore file type 过滤。 |
| `read_file` | 单个文件或单个 `pueue-log:<id>` handle | 支持 `start:stop` 行范围、负数尾部相对端点、行号、sampled inspection 和横向裁切。 |
| `search_text` | ripgrep-compatible 文本搜索 | 接受单个 path、多个 path 或 `pueue-log:<id>`；根据 `rg_args` 推断 grouped、context、counts、files、raw text 或 raw JSON 输出。 |
| `structural_search` | AST search session | 运行 `ast-grep` pattern/rule query，并注册 result group，供 `expand`、`around`、`explain_ast` 继续使用。 |
| `wait_pueue` | Pueue wait snapshot | 等待显式 task id 或当前未完成 task 快照，并返回可复用的 `pueue-log:<id>` handle。 |
| `apply_patch` | patch-shaped 文件修改程序 | 支持 Add/Delete/Update/Move file operation、connected file group、独立 group 之间的 partial success、warning block 和 repair artifact。 |
| `apply_rewrite` | numbered-selection-locked rewrite 程序 | 用行号加 visible text 选择旧跨度，然后删除或替换为 authored text。 |
| `apply_splice` | existing-span transfer 程序 | 对已经存在的文本跨度执行 copy、move、delete、insert、append 或 replace。 |

本地 CLI 覆盖同一组操作族：`list`、`read`、`search`、`wait-pueue`、`patch`、`rewrite`、`splice`。`set_workspace` 和 `structural_search` 是 MCP-facing 工具。

## 编辑模型

DocuTouch 保留三种独立编辑工具，因为它们表达的对象不同。

| Tool | 适用场景 | old-side evidence | 新文本 authoring |
| --- | --- | --- | --- |
| `apply_patch` | 你要把文件改成一个新的文本状态。 | patch context，可选 numbered `@@` header anchor。 | 通过 patch hunk 或 added file author。 |
| `apply_rewrite` | 你要替换或删除特定既有跨度，并希望选择器证据被锁定。 | numbered selection lines，可选 `... lines omitted ...`。 | 在 `*** With ... *** End With` 中逐字 author。 |
| `apply_splice` | 你要搬运、复制、删除或用已有文本替换已有文本。 | numbered source 与 target selection。 | splice 程序内不 author 新正文；结果文本来自 selected source span。 |

这个分离对 agent 可靠性很关键。Patch 适合生成新的文本状态。Rewrite 适合先证明旧跨度再 author replacement。Splice 适合目标内容已经在仓库里、操作本质是转移或复用的情况。

## `apply_patch`

`apply_patch` 是 patch-shaped 结构化写入工具。它接受熟悉的 patch envelope：

```text
*** Begin Patch
*** Update File: src/app.py
@@
-print("Hi")
+print("Hello")
*** End Patch
```

支持的 file operation 包括：

- `*** Add File: <path>`
- `*** Delete File: <path>`
- `*** Update File: <path>`
- `*** Move to: <new path>`，位于 update header 之后

当普通上下文不足以唯一定位时，可以在 hunk header 上放一个 numbered anchor：

```text
@@ 120 | def handler():
```

默认 numbered-evidence mode 是 `header_only`：numbered `@@` header 会被解释，而正文里恰好长成 `121 | value = 1` 的内容仍按普通 patch text 处理。高级 `full` mode 可在操作者显式开启时用于 dense old-side numbered evidence。

DocuTouch runtime 在 upstream patch shape 之外增加了这些行为：

- connected file group 内部原子提交
- disjoint file group 之间可以产生 `PartialSuccess`
- 成功结果可以携带 warning block，而不转成 failure
- 失败 patch 可以持久化到 `.docutouch/failed-patches/`
- diagnostics 会列出 committed changes、failed groups、attempted changes 和 repair hints

warning 会渲染成带 code 的 block：

```text
Success. Updated the following files:
M notes.md

warning[ADD_REPLACED_EXISTING_FILE]: Add File targeted an existing file and replaced its contents
  --> notes.md
  = help: prefer Update File when editing an existing file
```

partial failure 以稳定 error code 开头：

```text
error[PARTIAL_UNIT_FAILURE]: patch partially applied
```

输出随后会区分哪些已经落盘，哪些仍需要修复。

相关文档：

- [codex-apply-patch/README.md](codex-apply-patch/README.md)
- [codex-apply-patch/UPSTREAM_LINEAGE.md](codex-apply-patch/UPSTREAM_LINEAGE.md)
- [codex-apply-patch/LOCAL_DIVERGENCES.md](codex-apply-patch/LOCAL_DIVERGENCES.md)

## `apply_rewrite`

`apply_rewrite` 是 selection-locked edit 的 rewrite-program 工具。每个 rewrite action 先用 absolute line number 和完整 visible line content 选择一个既有连续跨度，然后删除这个跨度，或用 literal authored text 替换它。

最小的单行替换形态：

```text
*** Begin Rewrite
*** Update File: src/app.py
@@ replace the selected print line
12 | print("Hi")
*** With
print("Hello")
*** End With
*** End Rewrite
```

删除 action 以 `*** Delete` 结束：

```text
*** Begin Rewrite
*** Update File: src/obsolete.py
@@ remove the obsolete helper
40 | def legacy_helper():
... lines omitted ...
47 |     return cached_value
*** Delete
*** End Rewrite
```

`apply_rewrite` 也支持 file-level add、delete、update、move，但它的核心特征是 numbered selection contract。当模型应该先证明自己触碰的是哪一段旧文本，再 author replacement 时，这就是更合适的工具。

完整工具契约：

- [docutouch-server/tool_docs/apply_rewrite.md](docutouch-server/tool_docs/apply_rewrite.md)

## `apply_splice`

`apply_splice` 把既有文本跨度作为 transfer object。一个 splice program 选择 source span，然后对这个 span 执行 append、insert、replace、move、copy 或 delete。

最小 copy-append 程序：

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

选择器由 absolute line number 和 visible content 双重锁定。Omission marker 是语法的一部分，不是自然语言占位符：

- source selection 使用 `... source lines omitted ...`
- target selection 使用 `... target lines omitted ...`

当目标内容已经存在于某处，操作重点是 transfer relation 而不是新文本生成时，使用 `apply_splice`。

完整工具契约：

- [docutouch-server/tool_docs/apply_splice.md](docutouch-server/tool_docs/apply_splice.md)

## 阅读与搜索工作流

DocuTouch 的读取路径刻意保持文件边界。大量上下文发现的默认路径是：

1. 用 `list_directory` 建立文件地图。
2. 用 `search_text` 或 `structural_search` 缩小相关 path。
3. 对具体文件和行范围调用 `read_file`。
4. 证据稳定后，再使用 `apply_patch`、`apply_rewrite` 或 `apply_splice`。

`read_file` 的 `line_range` 使用 `start:stop` 形式。正数端点按 1-indexed line number 解释。任一端点都可以省略。负数端点按文件尾部相对位置解释。

示例：

```text
:50     前 50 行
50:     第 50 行到 EOF
-50:    最后 50 行
:-1     除最后一行外的全部内容
```

Sampled inspection 与 line range 是两套机制。当你需要稀疏局部视图，而不是一个连续范围时，使用 `sample_step` 和 `sample_lines`。

`search_text` 是智能 ripgrep 入口。`rg_args` 接受普通 ripgrep flags，工具会推断结果 surface：

| `rg_args` 意图 | 输出 surface |
| --- | --- |
| 默认搜索 | grouped results |
| `-A`、`-B`、`-C` | grouped context |
| `-c`、`--count`、`--count-matches` | counts |
| `-l`、`--files-with-matches`、`--files` | file list |
| `--json` | raw JSON |
| 无法忠实包装的混合 layout | raw text |

`query_mode` 默认为 `auto`：如果 regex parse 失败，会回退到 literal search，除非调用方显式要求 regex mode。

## 结构搜索

`structural_search` 使用 `ast-grep` executable。它通过 MCP 暴露，支持这些 mode：

| Mode | 用途 |
| --- | --- |
| `find` | 运行 pattern 或 rule query。 |
| `expand` | 展开已注册 result group。 |
| `around` | 查看 result reference 或 `path:line` 周围的局部结构上下文。 |
| `explain_ast` | 解释 result reference 或 `path:line` 的局部 AST shape。 |
| `rule_test` | 在小范围验证 ast-grep pattern 或 rule。 |

Result group 注册在当前 MCP connection 内。`ref="2"` 指最近一次 query 的 group `[2]`；`ref="q1.2"` 指 query `q1` 的 group `[2]`。

Rule 必须以 JSON object 传入。`fix`、`rewrite`、`replacement`、`apply`、`autofix`、`transform` 等 edit-producing 字段会被拒绝，因为 DocuTouch 把 structural search 用作证据工具，而不是 ast-grep 驱动的 mutation 工具。

完整工具契约：

- [docutouch-server/tool_docs/structural_search.md](docutouch-server/tool_docs/structural_search.md)

## 安装

### GitHub Releases

Release workflow 会从 `v*` tag 发布平台 binary：

- `docutouch-x86_64-pc-windows-msvc.exe`
- `docutouch-x86_64-unknown-linux-gnu`
- `SHA256SUMS.txt`

下载入口：

- [GitHub Releases](https://github.com/MichengLiang/docutouch/releases)

在 Linux 或 WSL 上，Linux path 应该配 Linux binary。即使你能从 WSL 启动 Windows `.exe`，这个进程仍然使用 Windows filesystem semantics，所以 `/home/user/project` 这样的 workspace 不会被当成合法 Windows path。

### 从源码构建

前提：

- Rust toolchain 和 Cargo
- `search_text` 需要 `rg` 在 `PATH` 上
- `structural_search` 需要 `ast-grep` 在 `PATH` 上
- `wait_pueue` 需要 `pueue` 和运行中的 `pueued` service

构建 workspace：

```bash
cargo build
```

构建 release binary：

```bash
cargo build --locked --release -p docutouch-server
```

编译出来的 binary 名为 `docutouch`，因为 `docutouch-server/Cargo.toml` 定义了这个 binary target。

### npm launcher

`npm/` 目录中的包是名为 `docutouch` 的薄 launcher。

```bash
npx docutouch help
npm install -g docutouch
```

这个 launcher 支持 Windows x64 和 Linux x64。第一次运行时，它会下载与 npm package version 匹配的 GitHub Release asset，并把 binary 缓存在已安装 package 的 `vendor/` 目录下。

npm dist-tag 可能落后于 GitHub Releases。如果 `npm view docutouch version` 比你想使用的 GitHub release 更旧，请直接下载 GitHub asset 或从源码构建。

## MCP 配置

对于采用常见 stdio 配置形态的 MCP 宿主，尽量直接指向编译好的 binary：

```json
{
  "mcpServers": {
    "docutouch": {
      "command": "/absolute/path/to/docutouch",
      "env": {
        "DOCUTOUCH_DEFAULT_WORKSPACE": "/absolute/path/to/project",
        "DOCUTOUCH_APPLY_PATCH_NUMBERED_EVIDENCE_MODE": "header_only"
      }
    }
  }
}
```

本地开发时，用 `cargo run` 也可以：

```json
{
  "mcpServers": {
    "docutouch": {
      "command": "cargo",
      "args": ["run", "-q", "-p", "docutouch-server"],
      "env": {
        "DOCUTOUCH_DEFAULT_WORKSPACE": "/absolute/path/to/project"
      }
    }
  }
}
```

通过 npm：

```json
{
  "mcpServers": {
    "docutouch": {
      "command": "npx",
      "args": ["-y", "docutouch"],
      "env": {
        "DOCUTOUCH_DEFAULT_WORKSPACE": "/absolute/path/to/project"
      }
    }
  }
}
```

如果宿主会在连接后立刻调用 `set_workspace`，`DOCUTOUCH_DEFAULT_WORKSPACE` 可以省略。

## CLI 用法

裸 `docutouch` 会启动 stdio MCP server。

```bash
docutouch
docutouch mcp
docutouch serve
```

本地 CLI 通过顶层 subcommand 暴露：

```bash
docutouch list [path] [--max-depth N] [--show-hidden] [--include-gitignored]
docutouch read <path> [--line-range START:END] [--show-line-numbers]
docutouch search <query> <path> [more_paths...] [--rg-args '...']
docutouch wait-pueue [TASK_ID ...] [--mode any|all] [--timeout-seconds N]
docutouch patch [patch-file] [--numbered-evidence-mode header_only|full]
docutouch rewrite [rewrite-file]
docutouch splice [splice-file]
```

从源码运行时，在同一组命令前加 `cargo run -p docutouch-server --`：

```bash
cargo run -p docutouch-server -- list docutouch-server/src
cargo run -p docutouch-server -- read README.md --line-range 1:40 --show-line-numbers
cargo run -p docutouch-server -- search apply_patch docutouch-server/src --view full
cargo run -p docutouch-server -- wait-pueue 42 --mode any
cargo run -p docutouch-server -- read pueue-log:42
cargo run -p docutouch-server -- patch retry.patch
cargo run -p docutouch-server -- rewrite rewrite.txt
cargo run -p docutouch-server -- splice splice.txt
```

`patch`、`rewrite`、`splice` 在没有 file argument 时会从 stdin 读取输入文本。

```bash
cat retry.patch | cargo run -p docutouch-server -- patch
cat rewrite.txt | cargo run -p docutouch-server -- rewrite
cat splice.txt | cargo run -p docutouch-server -- splice
```

对于 `.docutouch/failed-patches/*.patch` repair artifact，`patch` CLI 会在 replay 前自动恢复其所属 workspace anchor。

## 环境变量

| 变量 | 作用范围 | 含义 |
| --- | --- | --- |
| `DOCUTOUCH_DEFAULT_WORKSPACE` | MCP server | relative path 的默认 base path。 |
| `DOCUTOUCH_APPLY_PATCH_NUMBERED_EVIDENCE_MODE` | MCP 与 CLI patch runtime | `header_only` 或 `full`；CLI `--numbered-evidence-mode` 可以为单次 patch invocation 覆盖它。 |
| `DOCUTOUCH_PUEUE_BIN` | `wait_pueue` | `pueue` executable 的 path 或 command name。默认是 `pueue`。 |
| `DOCUTOUCH_PUEUE_RUNTIME_DIR` | `wait_pueue` | 覆盖 Pueue state/runtime directory resolution。 |
| `DOCUTOUCH_PUEUE_TIMEOUT_SECONDS` | `wait_pueue` | invocation 未传 `timeout_seconds` 时使用的默认等待超时。 |
| `PUEUE_CONFIG_PATH` | `wait_pueue` | runtime path discovery 使用的原生 Pueue config path。 |

## GitHub Release 流程

DocuTouch 使用 GitHub 作为公开分发和自动化表面。

Release workflow 在匹配 `v*` 的 tag 上运行：

1. Checkout repository。
2. 安装 stable Rust。
3. 在 Windows 和 Ubuntu runner 上以 release mode 构建 `docutouch-server`。
4. Stage 平台对应 binary asset。
5. Upload artifact。
6. 生成 `SHA256SUMS.txt`。
7. 发布 GitHub Release，并使用 generated release notes。

npm workflow 在 `release` workflow 于 `v*` tag 上成功完成后运行，也可以通过显式 tag 手动 dispatch：

1. 从 `workflow_run.head_branch` 或手动传入的 tag checkout release tag。
2. 安装 Node 24。
3. 校验 `v${npm/package.json.version}` 与 release tag 一致。
4. 在 `npm/` 下运行 `npm publish --provenance --access public`。

因此一次健康的公开 release 应该让三处版本一致：Git tag、`docutouch-server/Cargo.toml`、`npm/package.json`。

## 项目地图

| Path | 角色 |
| --- | --- |
| [docutouch-core/](docutouch-core/) | 共享 Rust library，提供 filesystem surface、search、rewrite/splice runtime、patch presentation 和 structural search。 |
| [docutouch-server/](docutouch-server/) | stdio MCP server 与本地 CLI adapter 的 binary crate。 |
| [docutouch-server/tool_docs/](docutouch-server/tool_docs/) | MCP server 内嵌的 tool descriptions。 |
| [codex-apply-patch/](codex-apply-patch/) | vendored patch runtime fork，包含 upstream lineage notes。 |
| [npm/](npm/) | 面向 GitHub Release binaries 的 Node launcher package。 |
| [docs/guide/](docs/guide/) | quickstart、CLI、MCP、tool surfaces 等 reader-facing guide。 |
| [docs/source/](docs/source/) | 更正式的 source documentation 与 knowledge records。 |
| [example/](example/) | 小示例与 MCP inspection scripts。 |
| [script/](script/) | 本地 helper scripts 与 demo。 |

## 文档入口

建议从这里开始：

- [docs/guide/quickstart.md](docs/guide/quickstart.md)
- [docs/guide/tool-surfaces.md](docs/guide/tool-surfaces.md)
- [docs/guide/mcp-server.md](docs/guide/mcp-server.md)
- [docs/guide/cli.md](docs/guide/cli.md)
- [docs/guide/apply-patch-and-apply-splice.md](docs/guide/apply-patch-and-apply-splice.md)
- [docutouch-server/README.md](docutouch-server/README.md)
- [docutouch-core/README.md](docutouch-core/README.md)
- [codex-apply-patch/README.md](codex-apply-patch/README.md)

当 contract、diagnostic surface、CLI 参数或 public tool boundary 发生变化时，应同时更新对应 embedded tool doc 和 reader-facing guide。

## 开发

运行主 workspace 测试：

```bash
cargo test -p docutouch-core -p docutouch-server
```

只运行 server smoke tests：

```bash
cargo test -p docutouch-server
```

运行 vendored patch engine 测试：

```bash
cd codex-apply-patch
cargo test
```

有用的检查命令：

```bash
cargo run -q -p docutouch-server -- help
cargo metadata --no-deps --format-version 1
npm view docutouch version dist-tags --json
```

## Contributing

修改 public behavior 前请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。简版原则是：

- PR 保持聚焦
- 说明改动是否影响 CLI、MCP、diagnostics 或 tool contracts
- public surface 变化时同步更新文档
- 保持 patch、rewrite、splice、search、read 工具之间的对象边界
- 行为变化要补测试；CLI/MCP surface 变化要有 smoke coverage

## 状态

主要 MCP 与 CLI 工具面已经可用。这个项目仍然是一个快速演进的 workbench：contract 已经成文，release automation 已经存在，public packaging 已经接上，但文档和产品边界仍会随着工具族扩展继续收口。

## License

Apache-2.0。见 [LICENSE](LICENSE)。
