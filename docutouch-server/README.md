# docutouch-server

`docutouch-server` 是 DocuTouch Rust 工作区里的工具服务入口 crate。

它现在提供两种对外使用方式：

- stdio MCP server
- 本地 CLI adapter

这两个入口共享同一套 core 语义、transport shell 与 tool-specific adapter 层：

- stdio MCP 通过 `ToolService` 暴露工具、workspace 与路径解析策略
- `apply_patch` / `apply_splice` 通过 shared transport shell 与各自 adapter 在 CLI / MCP 之间复用同一条调用路径
- `list` / `read` / `search` 的 CLI 仍直接调用共享 core 逻辑

## 当前暴露的工具

- `set_workspace`
- `list_directory`
- `read_file`
- `search_text`
- `apply_patch`
- `apply_splice`

## 目录结构

- `src/main.rs`
  程序入口。根据 CLI dispatch 决定启动 stdio server，或执行 CLI 子命令。

- `src/cli.rs`
  命令行分发与参数解析。当前支持：
  - `docutouch serve`
  - `docutouch list ...`
  - `docutouch read ...`
  - `docutouch search ...`
  - `docutouch patch ...`
  - `docutouch splice ...`

- `src/tool_service.rs`
  共享服务层。负责：
  - MCP 工具目录与 schema 暴露
  - JSON 参数解析
  - workspace/path 归一化
  - 调用 `docutouch-core` / shared transport shell / tool-specific adapter 执行实际工作

- `src/patch_adapter.rs`
  patch 调用适配层；建立在 shared transport shell 之上。

- `src/splice_adapter.rs`
  splice 调用适配层；建立在 shared transport shell 之上。

- `src/transport_shell.rs`
  patch/splice 共用的 CLI/MCP transport shell substrate。负责 generic anchor 选择、
  file-vs-stdin provenance 与通用 path 读取机械层。

- `src/server.rs`
  stdio MCP adapter。负责把 rmcp request 转成 `ToolService` 调用。

- `tool_docs/`
  工具文档字符串来源。

- `tests/`
  crate 层回归测试。覆盖 CLI / stdio 行为一致性与 `apply_patch` / `apply_splice` parity。

## 启动方式

### 1. stdio MCP server

```bash
cargo run -p docutouch-server
```

或：

```bash
cargo run -p docutouch-server -- serve
```

### 2. CLI adapter

```bash
cargo run -p docutouch-server -- list docutouch-server/src
cargo run -p docutouch-server -- read README.md --line-range 1:40
cargo run -p docutouch-server -- search apply_patch docutouch-server/src --view full
cat fix.patch | cargo run -p docutouch-server -- patch
cargo run -p docutouch-server -- patch .docutouch/failed-patches/1712345678901-0.patch
```

`splice` 也可以从 stdin 读取：

```bash
Get-Content move.splice | cargo run -p docutouch-server -- splice
```

### Patch Retry Artifact Workflow

`patch` 的默认相对路径锚点仍然是调用时 CWD。

但当 CLI 以 file-backed source 读取一个位于 `<workspace>/.docutouch/failed-patches/*.patch` 下的 failed patch artifact 时，
会自动把 execution/display anchor 恢复到该 artifact 所属的 workspace root。

这意味着：

- ordinary patch file 继续相对调用时 cwd 执行；
- DocuTouch 自己生成的 failed patch artifact 可以被直接编辑后重放；
- 重放失败时，diagnostics 继续指向原 patch file，而不是再生成第二层 failed-patches 副本。

典型 repair loop 是：

```powershell
Get-Content -Raw .docutouch\failed-patches\1712345678901-0.patch |
  cargo run -p docutouch-server -- patch

cargo run -p docutouch-server -- patch .docutouch\failed-patches\1712345678901-0.patch
```

推荐仍然从目标 workspace 内执行命令；
但如果显式传入的是 failed patch artifact file path，CLI 会按上述规则恢复其 workspace anchor。

## 测试

```bash
cargo test -p docutouch-server
```

当前测试覆盖：

- CLI 与 stdio 行为一致性
- `apply_patch` / `apply_splice` parity
- splice failure source-path presentation

## 相关文档

- [../README.md](../README.md)
  Rust 工作区总览。
