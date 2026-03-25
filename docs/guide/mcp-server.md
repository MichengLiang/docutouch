# MCP Server

`docutouch-server` 是当前工作区里的工具服务入口 crate。

它提供两种入口：

- 无参数启动：stdio MCP server
- 带子命令启动：CLI adapter

## 启动方式

cargo run -p docutouch-server -- serve
```

如果直接运行裸命令：

```bash
cargo run -p docutouch-server
```

当前会打印 CLI usage；真正启动 stdio MCP server 请使用 `serve`。

如果你使用 npm launcher，也可以：

```bash
npx @michengliang/docutouch serve
```

## 默认 workspace

如果你希望在启动时预设默认 workspace：

```bash
DOCUTOUCH_DEFAULT_WORKSPACE=/absolute/path/to/project cargo run -p docutouch-server -- serve
```

如果你还希望 server process 默认把 `apply_patch` 的 numbered-evidence mode 设为 `full`：

```bash
DOCUTOUCH_DEFAULT_WORKSPACE=/absolute/path/to/project DOCUTOUCH_APPLY_PATCH_NUMBERED_EVIDENCE_MODE=full cargo run -p docutouch-server -- serve
```

相对路径的解析优先使用显式 `set_workspace`，其次使用启动时有效的 `DOCUTOUCH_DEFAULT_WORKSPACE`。

## 最小配置示例

```json
{
  "command": "cargo",
  "args": ["run", "-q", "-p", "docutouch-server"],
  "env": {
    "DOCUTOUCH_DEFAULT_WORKSPACE": "/absolute/path/to/project",
    "DOCUTOUCH_APPLY_PATCH_NUMBERED_EVIDENCE_MODE": "header_only"
  }
}
```

通过 npm launcher 的最小配置示例：

```json
{
  "command": "npx",
  "args": ["-y", "@michengliang/docutouch"],
  "env": {
    "DOCUTOUCH_DEFAULT_WORKSPACE": "/absolute/path/to/project",
    "DOCUTOUCH_APPLY_PATCH_NUMBERED_EVIDENCE_MODE": "header_only"
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
