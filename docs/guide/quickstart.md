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

## 启动 stdio MCP server

```bash
cargo run -p docutouch-server
```

也可以显式写成：

```bash
cargo run -p docutouch-server -- serve
```

如果要在启动时提供默认 workspace：

```bash
DOCUTOUCH_DEFAULT_WORKSPACE=/absolute/path/to/project cargo run -p docutouch-server
```

如果你希望 server process 默认把 `apply_patch` 的 numbered-evidence mode 设为 `full`，可以再加：

```bash
DOCUTOUCH_DEFAULT_WORKSPACE=/absolute/path/to/project DOCUTOUCH_APPLY_PATCH_NUMBERED_EVIDENCE_MODE=full cargo run -p docutouch-server
```

## 最小 MCP 配置示例

```json
{
  "command": "cargo",
  "args": ["run", "-q", "-p", "docutouch-server"],
  "env": {
    "DOCUTOUCH_DEFAULT_WORKSPACE": "/absolute/path/to/project"
  }
}
```

## 直接调用 CLI

```bash
cargo run -p docutouch-server -- list docutouch-server/src
cargo run -p docutouch-server -- read README.md --line-range 1:40
cargo run -p docutouch-server -- search apply_patch docutouch-server/src --view full
```

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
