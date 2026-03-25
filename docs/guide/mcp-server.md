# MCP Server

`docutouch-server` 是当前工作区里的工具服务入口 crate。

它提供两种入口：

- 无参数启动：stdio MCP server
- 顶层工具子命令：本地 CLI adapter
- `mcp` / `serve`：显式 server alias

## 启动方式

```bash
cargo run -p docutouch-server
```

如果你想显式写出同一入口的 alias，也可以：

```bash
cargo run -p docutouch-server -- mcp
cargo run -p docutouch-server -- serve
```

如果你使用 npm launcher，也可以：

```bash
npx docutouch
```

## 默认 workspace

如果你希望在启动时预设默认 workspace：

```bash
DOCUTOUCH_DEFAULT_WORKSPACE=/absolute/path/to/project cargo run -p docutouch-server
```

如果你还希望 server process 默认把 `apply_patch` 的 numbered-evidence mode 设为 `full`：

```bash
DOCUTOUCH_DEFAULT_WORKSPACE=/absolute/path/to/project DOCUTOUCH_APPLY_PATCH_NUMBERED_EVIDENCE_MODE=full cargo run -p docutouch-server
```

相对路径的解析优先使用显式 `set_workspace`，其次使用启动时有效的 `DOCUTOUCH_DEFAULT_WORKSPACE`。

## 最小配置示例

如果你的宿主采用常见的 stdio MCP 配置格式，推荐直接指向编译好的二进制：

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

在 Windows 上，`command` 通常会是 `C:\\...\\docutouch.exe`。

如果你还在源码态调试，也可以继续用 `cargo run`：

```json
{
  "mcpServers": {
    "docutouch": {
      "command": "cargo",
      "args": ["run", "-q", "-p", "docutouch-server"],
      "env": {
        "DOCUTOUCH_DEFAULT_WORKSPACE": "/absolute/path/to/project",
        "DOCUTOUCH_APPLY_PATCH_NUMBERED_EVIDENCE_MODE": "header_only"
      }
    }
  }
}
```

通过 npm launcher 的最小配置示例：

```json
{
  "mcpServers": {
    "docutouch": {
      "command": "npx",
      "args": ["-y", "docutouch"],
      "env": {
        "DOCUTOUCH_DEFAULT_WORKSPACE": "/absolute/path/to/project",
        "DOCUTOUCH_APPLY_PATCH_NUMBERED_EVIDENCE_MODE": "header_only"
      }
    }
  }
}
```

如果你的宿主会在连接后立刻调用 `set_workspace`，也可以不设置默认 workspace。

仓库侧可以同时准备 npm trusted-publishing workflow，但 npm package settings 侧仍然需要完成 trusted publisher 绑定后，release 驱动的自动 npm publish 才会真正成功。

## 公开工具面

- `set_workspace`
- `list_directory`
- `read_file`
- `search_text`
- `apply_patch`
- `apply_splice`

模块级说明见 [../../docutouch-server/README.md](../../docutouch-server/README.md)。
