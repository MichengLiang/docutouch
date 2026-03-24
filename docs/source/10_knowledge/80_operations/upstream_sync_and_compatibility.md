(knowledge-operations-upstream-sync-and-compatibility)=
# Upstream Sync And Compatibility

## 作用域

本页记录 DocuTouch 当前对上游同步与兼容性维护的已接纳操作知识。

它回答：

- 上游同步时要区分哪些基线
- 本地分叉何时必须显式记录
- 兼容性行为如何被披露而不被鼓励

## 上游同步的三分法

维护 `codex-apply-patch` 相关能力时，应始终区分三件事：

1. 上游基线是什么
2. 本地已确认增强是什么
3. 当前争议点与本地立场是什么

这三者不得混写成单一“实现现状”。

## Internal Substrate Boundary

一旦某个 upstream-derived crate 已作为 vendored fork 纳入 DocuTouch 仓库，
就应同时区分两件事：

1. 它作为上游比较基线时的披露义务
2. 它作为当前仓库内部 correctness substrate 时的架构地位

对于 `codex-apply-patch`，当前 accepted posture 是：

- 继续保留 upstream baseline / local enhancement / controversy 的显式记录纪律；
- 但 downstream tool architecture 不再受“保持靠近上游”这一点本身支配；
- 当 `apply_splice` 或其他 DocuTouch-owned surface 需要更健康的 shared substrate 时，应优先按本仓库的 correctness 与 maintainability 判断来做结构调整。

因此：

- upstream 仍是 disclosure object
- vendored fork 同时也是 internal substrate
- internal substrate 可以被 extract、reframe 或重组，只要相关 divergence 被真实记录

## 分叉记录纪律

若某次改动会让 DocuTouch 在语义上进一步偏离上游基线，
则必须显式记录：

- 为什么偏离
- 偏离在哪里
- 代价是什么

不得让分叉只停留在实现或聊天结论里。

## 兼容性披露原则

兼容性行为不等于推荐行为。

若某个行为因为兼容上游或模型分布而被保留，
文档与 UX 应优先做到：

- 披露真实行为
- 在触发时给出 warning
- 告诉使用者发生了什么与更推荐什么

而不是把兼容性行为写成常规技巧。

## 正确性优先级

下列问题优先视为 correctness hardening，
而不是风格性微调：

- 路径同一性
- Windows case alias
- same-path move
- workspace / path drift
- 原子性与回滚缺陷

## Source Basis

- `docs/archive2026年3月24日/maintainer_guide.md`
