# docutouch-server

`docutouch-server` 是 DocuTouch Rust 工作区里的工具服务入口 crate。

它当前提供两种外部入口：

- stdio MCP server
- 本地 CLI adapter

这两个入口复用同一套 core 语义与 transport shell。

## 当前公开工具面

- `set_workspace`
- `list_directory`
- `read_file`
- `search_text`
- `apply_patch`
- `apply_splice`

## 启动方式

启动 stdio MCP server：

```bash
cargo run -p docutouch-server
```

显式 server alias 也仍然可用：

```bash
cargo run -p docutouch-server -- mcp
cargo run -p docutouch-server -- serve
```

裸命令现在就是默认 stdio MCP server 入口。

直接调用 CLI：

```bash
cargo run -p docutouch-server -- list docutouch-server/src
cargo run -p docutouch-server -- read README.md --line-range 1:40
cargo run -p docutouch-server -- search apply_patch docutouch-server/src --view full
```

`patch` 与 `splice` 也可以从 stdin 读取输入文本。

对于 `patch`，CLI 同时支持 file-backed repair loop。当前行为包括：

- 普通 patch file 继续相对调用时的 CWD 执行
- `.docutouch/failed-patches/*.patch` 这类 repair artifact file 会恢复其所属 workspace anchor
- 重放失败时，diagnostics 继续指向当前 patch file

## 代码入口

- `src/main.rs`
  程序入口。

- `src/cli.rs`
  CLI 分发与参数解析。

- `src/tool_service.rs`
  共享服务层，负责工具目录、schema、workspace/path 归一化与调用下层执行逻辑。

- `src/patch_adapter.rs`
  patch 调用适配层。

- `src/splice_adapter.rs`
  splice 调用适配层。

- `src/transport_shell.rs`
  patch/splice 共用的 transport shell substrate。

- `src/server.rs`
  stdio MCP adapter。

## 测试

```bash
cargo test -p docutouch-server
```

## 相关文档

- [../README.md](../README.md)
- [../docs/README.md](../docs/README.md)
- [../docutouch-core/README.md](../docutouch-core/README.md)
