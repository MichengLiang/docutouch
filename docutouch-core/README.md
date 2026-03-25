# docutouch-core

`docutouch-core` 是 DocuTouch Rust 工作区里的共享原语层。

它当前承担的对象包括：

- 目录树列出与 ASCII 渲染
- 单文件读取与相关渲染语义
- 基于 ripgrep 的分组搜索包装
- `apply_patch` / `apply_splice` 结果映射与文本呈现

更高层的工具入口在：

- [../README.md](../README.md)
- [../docutouch-server/README.md](../docutouch-server/README.md)
