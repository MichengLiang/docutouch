# codex-apply-patch

`codex-apply-patch` 是 `openai/codex` 仓库中 `codex-rs/apply-patch` 子树的 vendored fork。

在当前工作区里，它承担 DocuTouch 的底层 patch 执行能力。

## 在整个项目中的角色

- 保留 upstream `apply-patch` 的输入形态
- 承载 DocuTouch 当前的 patch runtime 行为
- 为 `docutouch-core` 与 `docutouch-server` 提供底层 patch 执行能力

主要的运行时差异集中在：

- connected file groups
- `PartialSuccess`
- 便于继续修复的 diagnostics
- source-location-aware failure metadata
- warning blocks for compatibility behavior

## 相关文档

- [UPSTREAM_LINEAGE.md](UPSTREAM_LINEAGE.md)
  上游来源、baseline commit 与同步姿态。

- [LOCAL_DIVERGENCES.md](LOCAL_DIVERGENCES.md)
  当前相对 upstream 的本地分叉对象。

## 测试

```bash
cargo test
```
