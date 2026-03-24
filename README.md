# DocuTouch Rust 工作区

本目录是 DocuTouch 当前正在活跃开发的 Rust 实现版本。

当前的系统架构被组织为一个精简的 Rust MCP（模型上下文协议）技术栈：

- `codex-apply-patch`
  这是 OpenAI Codex `apply-patch` 核心逻辑的一个本地内嵌（vendored）分支。
  其解析器与补丁写入模型严格保持与 Codex 基线一致。
  但在当前仓库内，它同时被视为 DocuTouch 的 internal correctness substrate，
  可以在保持 divergence disclosure 真实记录的前提下继续被抽取、重组与复用。
  此处最核心的本地增强在于**更强大的文件组提交模型（file-group commit model）**：存在关联的文件操作将作为一个文件组进行原子化提交；而相互独立的文件组，即使其他组发生失败，依然可以成功被应用（Partial Success）。

- `docutouch-core`
  供 MCP server 与 CLI adapter 共享的文件系统原语、搜索包装与 patch 文本渲染层。
  当前主要职责：
  - 生成 ASCII 树状目录列表
  - 控制隐藏文件 / `.gitignore` 忽略文件的可见性（可选）
  - 在目录输出中附加时间戳字段（可选）
  - 支持单文件读取（可选附带基于 1 索引的绝对行号）
  - 提供 `search_text` 的共享分组搜索与渲染语义
  - 为 MCP 输出提供结构化的 `apply-patch` 执行结果映射
  - 提供 CLI / MCP 共用的 patch success / failure 文本呈现

- `docutouch-server`
  纯粹使用 Rust 编写的工具服务入口 crate，同时提供 stdio MCP 和 CLI adapter。
  当前对外暴露的 MCP 工具（Tools）：
  - `set_workspace`（设置工作区）
  - `list_directory`（列出目录）
  - `read_file`（读取单文件）
  - `search_text`（基于 ripgrep 的分组搜索）
  - `apply_patch`（应用补丁）
  - `apply_splice`（搬运既有文本跨度）

  当前 `docutouch` 二进制在 transport 层提供两种入口：
  - 无参数启动：stdio MCP server
  - CLI subcommands：`docutouch list/read/search/patch/splice`

  说明：
  - `read_files` 已在 Wave 0.5 中完成退役，server / core 的可调用实现已删除。
  - `docutouch-server/tool_docs/read_files.md` 现在作为退役记录保留，说明其历史目的、移除原因，以及为什么推荐重复调用普通 `read_file`。
  - CLI adapter 仍然是次级形态；MCP / 注入式接口仍然是主产品 surface。
  - `apply_patch` 与 `apply_splice` 现在通过 `docutouch-server` 内部的 shared transport shell 与各自 tool-specific adapter 在 CLI / MCP 之间复用同一条调用路径；`list/read/search` 仍以共享 core 语义为主。

## 当前输出约定 (Output Conventions)

- `list_directory` 始终保持以 ASCII 树作为主要形态。
- 默认显示文件大小和总行数。
- 时间戳显示默认为关闭，需通过 `timestamp_fields` 参数显式开启（opt-in）。
- `read_file` 默认返回纯文本内容，并可选择开启基于 1 索引的行号显示。
- 任一 sampled 参数出现时，`read_file` 会进入 sampled local inspection mode；缺省 sampled 参数会补稳定默认值，但不会隐式启用横向裁切。
- 大量文件阅读应在编排层重复调用 `read_file`，保持文件边界稳定，并按需拆分 `line_range`；不再推荐把多文件正文聚合成一次巨型返回。
- `search_text` 使用 ripgrep 做底层搜索，但把结果按文件分组返回，减少重复路径噪音，并让下一步 `read_file` 更自然。

- `apply_patch` 在成功路径保持 Codex 风格的 `A/M/D` 摘要。
- `apply_patch` 在部分成功路径会同时报告：
  - 已提交的 `committed changes`
  - 失败的 `failed file groups`
  - 每组失败的 error code、target、action / hunk 与 attempted `A/M/D`

## 编译构建 (Build)

如果你需要预先编译可执行文件（例如为了打包部署或语法检查），请使用以下指令：

**1. 日常开发构建 (Debug 模式)**
此模式编译速度最快，包含完整的调试符号，但未进行深度性能优化。
```bash
cargo build
```
*注：由于使用了 Cargo Workspace 机制，此命令会自动编译栈内的所有 crate（core, server, apply-patch）。主二进制产物将位于 `target/debug/docutouch`。*

**2. 生产环境构建 (Release 模式)**
如果你准备将此 MCP 服务器接入实际的大语言模型工作流，或进行高并发性能测试，**强烈建议使用发布模式**。Rust 编译器将执行极限优化，运行时性能会有数量级的提升。
```bash
cargo build --release
```
*注：编译过程耗时较长。最终的发布模式二进制产物将位于 `target/release/docutouch`，你可以直接将其拷贝到任何目标环境运行。*

## 运行 Rust MCP 服务器

在开发过程中，你可以直接使用 `cargo run` 边编译边运行：

```bash
cargo run -p docutouch-server
```

也可以显式写成：

```bash
cargo run -p docutouch-server -- serve
```

如果希望在 server 启动时预设一个默认 workspace，可以在启动前设置：

```bash
DOCUTOUCH_DEFAULT_WORKSPACE=/absolute/path/to/project cargo run -p docutouch-server
```

路径优先级为：

1. 显式调用 `set_workspace`
2. 启动时存在且有效的 `DOCUTOUCH_DEFAULT_WORKSPACE`
3. 否则 relative path 直接报错，并提示调用 `set_workspace` 或改用 absolute path

说明：

- `apply_patch` 不再把 server process cwd 当作语义上的默认 workspace。
- absolute path 仍然始终可用。
- 如果 `DOCUTOUCH_DEFAULT_WORKSPACE` 存在但无效，server 仍会启动，并把它视为“没有默认 workspace”。
- 如果既没有显式 workspace，也没有有效的默认 workspace，那么相对路径 patch 会在 parse / path-resolution 阶段直接失败，并以内联 diagnostics 报告原因。此时应优先显式 `set_workspace`，或直接使用 absolute path。
- `apply_patch` 不再写入 audit-shaped failure reports、secondary JSON sidecars 或 patch-run caches。
- 当 patch 通过 inline 参数或 stdin 提供且执行失败时，运行时可以把 failed patch source 本身持久化到工作区隐藏目录，供后续模型修复读取。
- patch 失败后的审计与回执仍优先依赖 Codex 宿主本身的 tool-call logs；持久化的 patch source 属于修复对象，不是第二套审计工件。

## 运行 CLI Adapter

`docutouch` CLI 是 MCP 工具面的 adapter，而不是第二套语义层。

当前提供的子命令是：

- `docutouch list`
- `docutouch read`
- `docutouch search`
- `docutouch patch`

CLI 约定：

- relative path 一律以当前进程 CWD 作为隐式 workspace anchor
- absolute path 仍然始终可用
- `search` 复用 MCP 的 grouped `preview/full` 输出语义
- `patch` 复用 MCP 的 success / warning / failure diagnostics 语义
- `patch` 在未提供 patch file 参数时，从 stdin 读取 patch 文本
- 独立的 `apply_patch` 二进制继续保留；`docutouch patch` 是统一工具面的并存 adapter，而不是替代品

示例：

```bash
cargo run -p docutouch-server -- list docutouch-server/src
cargo run -p docutouch-server -- read README.md --line-range 1:40
cargo run -p docutouch-server -- read README.md --line-range 1:120 --sample-step 5
cargo run -p docutouch-server -- read README.md --line-range 1:120 --sample-step 5 --sample-lines 2 --max-chars 80
cargo run -p docutouch-server -- search apply_patch docutouch-server/src docutouch-core/src --view full
cat fix.patch | cargo run -p docutouch-server -- patch
```

## 测试 (Testing)

针对 Core 与 MCP Server 的回归测试：

```bash
cargo test -p docutouch-core -p docutouch-server
```

针对内嵌的 `apply-patch` 分支的回归测试：

```bash
cd codex-apply-patch
cargo test
```

## 上游基线说明 (Upstream Baseline Notes)

对于内嵌的 `codex-apply-patch` 分支，我们保留了显式的本地追踪说明文件：

- `codex-apply-patch/UPSTREAM_BASELINE.md`
- `codex-apply-patch/DOCUTOUCH_ENHANCEMENTS.md`

这两个文件详细记录了当前工作区所遵循的上游 Codex 基线版本，以及我们在本地运行时层面对哪些行为进行了刻意增强。

## 维护文档 (Maintainer Docs)

为了避免关键背景信息散落在即时聊天记录中，工作区内长期维护所需的定位、规则、计划与专项设计记录统一沉淀在：

- `docs/README.md`
- `docs/product_positioning.md`
- `docs/maintainer_guide.md`
- `docs/roadmap.md`
- `docs/apply_patch_semantics_plan.md`

如果你准备继续维护这个工作区，建议先从 `docs/README.md` 开始阅读。
