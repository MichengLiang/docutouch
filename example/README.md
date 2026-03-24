# DocuTouch Example Showcase

这个目录用于展示当前 DocuTouch Rust 实现的真实运行效果，尤其是
`apply_patch` 在成功与失败路径下的可见输出形态。

## 文件

- `1.py`
  一个完整的 MCP 演示脚本。
  它会自动连接本仓库里编译出来的 `docutouch.exe`，在
  `tmp/example_workspace` 下构造演示工作区，然后依次展示：
  - tool listing
  - `set_workspace`
  - `list_directory`
  - `read_file`
  - `apply_patch` full success
  - `apply_patch` empty patch
  - `apply_patch` outer invalid add line
  - `apply_patch` target missing
  - `apply_patch` context mismatch
  - `apply_patch` target-anchor mismatch
  - `apply_patch` partial success
  - `apply_patch` large partial success
  - `apply_patch` move write error

## 运行方式

建议先编译服务端：

```powershell
cargo build -p docutouch-server
```

如果想优先用 release：

```powershell
cargo build -p docutouch-server --release
```

然后运行示例：

```powershell
uv run python example/1.py
```

脚本会优先寻找：

1. `target/debug/docutouch.exe`
2. `target/release/docutouch.exe`

## 输出特点

这个脚本不是只验证“能不能跑通”，而是为了把输出直接打到终端，方便观察：

- 成功路径长什么样
- compact full failure 长什么样
- partial failure 长什么样
- target anchor 怎么显示
- failed patch source 持久化后，diagnostics 里会怎么引用 patch path

## 注意事项

- 脚本会在 `tmp/example_workspace` 下创建并清理自己的演示目录
  `__diagnostics_demo__`。
- stderr 日志默认落在：
  - `tmp/example_workspace/diagnostics_demo_stderr.log`
- 如果你想保留现场，可以临时注释掉 `local_cleanup()`。
