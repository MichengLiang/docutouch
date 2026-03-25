# CLI

`docutouch` CLI 是 MCP 工具面的 adapter。

如果你不从源码构建，也可以通过 npm launcher 获取同一个 CLI 入口：

```bash
npx docutouch list docutouch-server/src
```

当前正式 CLI 入口形态是：

- `docutouch list`
- `docutouch read`
- `docutouch search`
- `docutouch patch`
- `docutouch splice`

如果你想显式写出分组，`docutouch cli <subcommand>` 也仍然可用；它与顶层子命令执行的是同一套 CLI adapter。

## 常见命令

```bash
cargo run -p docutouch-server -- list docutouch-server/src
cargo run -p docutouch-server -- read README.md --line-range 1:40
cargo run -p docutouch-server -- search apply_patch docutouch-server/src --view full
```

`patch` 与 `splice` 都支持两种进入方式：

- 直接从 stdin 读取输入文本
- 显式传入 patch / splice file

`patch` 还支持：

- `--numbered-evidence-mode header_only|full`

默认是 `header_only`。如果一次性重放需要 dense body-level numbered old-side evidence，可以显式传 `full`。

如果后续发布了 npm wrapper `docutouch`，CLI 的目标入口也可以通过 `npx docutouch patch ...`、`npx docutouch read ...` 这类顶层子命令获得；其本质仍然是调用同一个 `docutouch` 二进制。

这让 CLI 可以承担一条直接的 repair loop：

1. patch 失败
2. 失败 patch source 被持久化到 `.docutouch/failed-patches/*.patch`
3. 直接编辑这个 patch 文件
4. 再次通过 CLI 重放

当传入的是 `.docutouch/failed-patches/*.patch` 这类 repair artifact file 时，CLI 会恢复它所属的 workspace anchor。

CLI 继续保留，但当前主要接入方式仍是 MCP / injection；裸 `docutouch` 命令本身保留给 stdio MCP server。
