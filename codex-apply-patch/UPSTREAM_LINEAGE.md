# Upstream Lineage

## Upstream Source

- upstream repository: `https://github.com/openai/codex`
- source subtree: `codex-rs/apply-patch`
- baseline branch at check time: `main`
- baseline commit checked on 2026-03-18: `a265d6043edc8b41e42ae508291f4cfb9ed46805`

## Sync Posture

当前本地 fork 继续以 upstream `apply-patch` 的输入形态为基础。

同步时重点比较的对象包括：

- parser 行为
- invocation parsing
- seek / match 行为
- runtime contract 的刻意分叉面

## Packaging Posture

当前本地 crate 以 standalone crate 方式保留，不继承 upstream workspace manifest。

upstream lineage 与本地 packaging 是两个不同对象：

- lineage 说明它来自哪里
- packaging 说明它如何在当前仓库中存在
