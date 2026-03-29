# Quick Start

## 前提

- Rust toolchain
- Cargo
- 如果要使用 `search_text`，确保 `rg`（ripgrep）在 PATH 中可用

当前公开安装路径是 source build。

## 构建

在仓库根目录执行：

```bash
cargo build
```

如果你不想从源码构建，也可以直接使用：

- GitHub Releases: `https://github.com/MichengLiang/docutouch/releases`
- npm launcher: `npx docutouch help`

如果当前 tag 已经有 GitHub Release，你也可以直接下载对应平台的 release binary，而不是先从 source build 开始。

## 启动 stdio MCP server

```bash
cargo run -p docutouch-server
```

如果你希望显式写出 server alias：

```bash
cargo run -p docutouch-server -- mcp
cargo run -p docutouch-server -- serve
```

裸命令现在就是主 stdio MCP server 入口。

如果要在启动时提供默认 workspace：

```bash
DOCUTOUCH_DEFAULT_WORKSPACE=/absolute/path/to/project cargo run -p docutouch-server
```

如果你希望 server process 默认把 `apply_patch` 的 numbered-evidence mode 设为 `full`，可以再加：

```bash
DOCUTOUCH_DEFAULT_WORKSPACE=/absolute/path/to/project DOCUTOUCH_APPLY_PATCH_NUMBERED_EVIDENCE_MODE=full cargo run -p docutouch-server
```

## 最小 MCP 配置示例

如果你的宿主采用常见的 stdio MCP 配置格式，推荐直接指向编译好的二进制：

```json
{
  "mcpServers": {
    "docutouch": {
      "command": "/absolute/path/to/docutouch",
      "env": {
        "DOCUTOUCH_DEFAULT_WORKSPACE": "/absolute/path/to/project"
      }
    }
  }
}
```

在 Windows 上，`command` 通常会是 `C:\\...\\docutouch.exe`。

如果你还在源码态调试，再用 `cargo run`：

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

如果通过 npm launcher 接入：

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

repo 里可以同时准备 npm trusted-publishing workflow，但 npm package settings 侧仍然需要把 trusted publisher 绑定完成后，自动发布才会真正生效。

## 直接调用 CLI

```bash
cargo run -p docutouch-server -- list docutouch-server/src
cargo run -p docutouch-server -- read README.md --line-range 1:40
cargo run -p docutouch-server -- search apply_patch docutouch-server/src --view full
cargo run -p docutouch-server -- wait-pueue 42 --mode any
cargo run -p docutouch-server -- read pueue-log:42
cargo run -p docutouch-server -- search ERROR pueue-log:42 --view preview
```

当 npm wrapper `docutouch` 首次发布后，也可以用：

```bash
npx docutouch list docutouch-server/src
```

如果启用了 Pueue integration，`wait-pueue` 返回的 `pueue-log:<id>` handle 可继续交给 `read` 或 `search`。

## 一个最短的 patch 调用

```bash
cat fix.patch | cargo run -p docutouch-server -- patch
```

如果某次 CLI 重放需要单次打开 dense numbered old-side evidence：

```bash
cargo run -p docutouch-server -- patch --numbered-evidence-mode full retry.patch
```

如果 patch 失败并已经被持久化到 `.docutouch/failed-patches/*.patch`，可以直接编辑后重放。

## 下一步

- 如果你要理解工具集合，阅读 [tool-surfaces.md](tool-surfaces.md)
- 如果你要接 MCP，阅读 [mcp-server.md](mcp-server.md)
- 如果你要从 CLI 使用，阅读 [cli.md](cli.md)
