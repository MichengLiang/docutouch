# Contributing

感谢关注 DocuTouch。

## 在开始之前

当前仓库仍在持续打磨公开文档与工具边界。提交改动前，请先确认你的改动属于下面哪一类：

- bug fix
- tests
- 文档修订
- 小范围行为改进
- 明确讨论过的功能工作

如果改动会改变工具 contract、diagnostics surface、上游兼容姿态或产品边界，请不要只改代码。

## 文档约定

当前仓库同时保留两层文档：

- `docs/source/`
  当前项目文档的权威依据

- `docs/guide/`
  面向仓库读者的投影文档

出现冲突时，以 `docs/source/` 中对应的正式对象为准。

如果你修改了：

- 工具 contract
- diagnostics 行为
- CLI / MCP 对外表面
- upstream lineage / divergence posture

请同时更新相应文档。

## 本地检查

在提交前，至少运行：

```bash
cargo check -p docutouch-core -p docutouch-server
```

以及：

```bash
cd codex-apply-patch
cargo check
```

如果你的改动涉及测试或边界行为，也请运行对应测试。

## Patch 与 Splice 边界

当前仓库刻意保留：

- `apply_patch`
- `apply_splice`

两个独立工具身份。

提交前请先确认你的改动操作的对象是什么：

- 文本差异与新文本状态，属于 `apply_patch`
- 既有文本跨度的复制、移动、删除、替换，属于 `apply_splice`

不要因为底层共享 substrate，就把两个工具的产品边界混写。

## Pull Request 期望

Pull Request 应尽量聚焦。

请在描述中写清楚：

- 你改了什么
- 为什么要改
- 是否影响对外行为
- 需要读哪些文档或测试来理解这次改动

如果改动只是局部文案修订，也请说明涉及哪些文件。
