# Upstream Lineage

`codex-apply-patch` 来自 `openai/codex` 仓库中的 `codex-rs/apply-patch` 子树。

当前记录的 baseline commit 是：

- `a265d6043edc8b41e42ae508291f4cfb9ed46805`

DocuTouch 当前相对 upstream 的主要差异集中在运行时对象层。最重要的几类对象是：

- 当前 runtime 引入了 connected file-group commit model
- 执行结果支持 `PartialSuccess`
- 失败对象带有更完整的 diagnostics metadata
- parser / diagnostics 层保留了更细的 source-location-aware 信息

这条 lineage 也不能被简化成“尽量保持靠近上游”。当前 accepted posture 更接近下面这组事实：

- upstream 继续承担来源与比较基线的角色
- vendored fork 同时也是当前仓库里的底层 patch 能力来源
- downstream tool architecture 不再受“尽量别动上游”本身支配
- 只要 divergence 被真实记录，internal substrate 可以继续被抽取、重组和复用

模块级入口见 [../../codex-apply-patch/README.md](../../codex-apply-patch/README.md)。更细的 disclosure 见：

- [../../codex-apply-patch/UPSTREAM_LINEAGE.md](../../codex-apply-patch/UPSTREAM_LINEAGE.md)
- [../../codex-apply-patch/LOCAL_DIVERGENCES.md](../../codex-apply-patch/LOCAL_DIVERGENCES.md)
