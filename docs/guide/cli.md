# CLI

`docutouch` CLI 是 MCP 工具面的 adapter。

它当前提供的子命令是：

- `docutouch list`
- `docutouch read`
- `docutouch search`
- `docutouch patch`
- `docutouch splice`

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

这让 CLI 可以承担一条直接的 repair loop：

1. patch 失败
2. 失败 patch source 被持久化到 `.docutouch/failed-patches/*.patch`
3. 直接编辑这个 patch 文件
4. 再次通过 CLI 重放

当传入的是 `.docutouch/failed-patches/*.patch` 这类 repair artifact file 时，CLI 会恢复它所属的 workspace anchor。

CLI 继续保留，但当前主要接入方式仍是 MCP / injection。
