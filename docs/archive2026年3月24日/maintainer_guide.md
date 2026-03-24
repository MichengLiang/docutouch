# DocuTouch Maintainer Guide

## 目的

这份文档面向未来的维护者，而不是面向工具使用者。它回答的问题是：

- 修改这个项目时，什么东西最容易被破坏？
- 哪些原则必须保持稳定？
- 哪些上下文必须写进文档，而不能留在脑子里？
- 如何避免项目在数个迭代后变成边界混乱的集合体？

## 维护总原则

### 1. 优先维护主路径，而不是追求功能数量

对于 DocuTouch，这条主路径是：

- 定位工作区
- 列目录
- 读取上下文
- 结构化 patch 写入
- 获取可诊断反馈

说明：

- 当前“读取上下文”的推荐实现方式是重复调用普通 `read_file`，而不是恢复一个会聚合巨量正文的批量读取工具。

如果一个新想法不能明显改善这条主路径，就应当谨慎评估，不要轻易引入。

### 2. 先问“这是不是基础层问题”，再问“这是不是一个新功能”

基础层问题通常包括：

- 路径正确性
- 事务边界
- 回滚一致性
- 输出结构稳定性
- 跨平台差异
- warning / error 的可解释性

这类问题通常比扩新命令更值得优先处理。

### 3. 不要把临时聊天结论当成长期事实

如果某个讨论结果会影响未来维护，至少应落到下面三类文档之一：

- 定位/边界文档
- 设计计划文档
- 维护规则文档

否则几周以后它就会失效，只剩下零散聊天记录。

## 文档维护规则

### 必须同步更新文档的情形

下面这些变化发生时，不能只改代码：

- 产品定位变化
- 主接口 contract 变化
- warning / error 策略变化
- 与上游基线的关系变化
- 工具边界变化
- 近期优先级变化
- 对 CLI / MCP / 注入式形态的判断变化

### 文档分类约定

- `README.md`
  面向首次进入工作区的人，给出总体结构与快速入口。

- `docs/product_positioning.md`
  记录“我们是什么、不是什么、为什么这么做”。

- `docs/maintainer_guide.md`
  记录维护原则与长期约束。

- `docs/roadmap.md`
  记录阶段目标、暂缓事项与优先级变化。

- 专项设计文档，例如 `apply_patch_semantics_plan.md`
  记录具体子系统的技术判断与方案。

### 推荐的写法

文档应该优先写：

- 为什么这样定
- 有哪些权衡
- 哪些是当前明确不做的
- 未来维护者最可能误判什么

文档不应该只写成：

- 功能列表
- 无背景的结果陈述
- 仅靠当下参与者才读得懂的暗示式笔记

## 上游同步原则

`codex-apply-patch` 是 vendored fork。维护时要始终区分三件事：

### 1. 上游基线是什么

看：

- `codex-apply-patch/UPSTREAM_BASELINE.md`

### 2. 我们已经确认的本地增强是什么

看：

- `codex-apply-patch/DOCUTOUCH_ENHANCEMENTS.md`

### 3. 当前争议点与本地立场是什么

看：

- `docs/apply_patch_semantics_plan.md`

维护者不要在没有记录的情况下默默扩大与上游的行为分叉。只要某次改动会让 DocuTouch 在语义上进一步偏离 OpenAI 基线，就必须把“为什么偏离、偏离在哪里、代价是什么”记录下来。

## 对 warning、error 与兼容性的维护原则

### 1. 兼容性行为不等于推荐行为

如果一个行为因为兼容上游或模型分布而被保留，不表示应当在文档和 UX 里鼓励模型经常这么做。

### 2. 能透明化，就不要静默

如果某个行为会成功，但具有风险或语义张力，应优先考虑：

- 文档披露
- triggered warning
- 结构化返回字段

而不是让它悄悄成功。

### 3. correctness 问题优先于风格偏好

像下面这些应优先视为 correctness hardening：

- 路径同一性
- Windows case alias
- same-path move
- workspace/path drift
- 原子性与回滚缺陷

这类问题通常比“要不要更严格地解释某个 patch header”更优先。

## 测试维护原则

### 1. 每次边界修复都应留下回归测试

如果某次修复的价值来自边界行为，就不要只改代码。必须补测试，避免未来回退。

典型边界包括：

- patch 语义边界
- 事务边界
- Windows 路径边界
- alias path 边界
- success-with-warning 场景

### 2. 三层测试分别承担不同职责

- `codex-apply-patch`
  底层语义、路径、提交模型与 warning 检测

- `docutouch-core`
  结构化映射与 outcome 表达

- `docutouch-server`
  端到端工具消息、MCP 接口与用户可见 UX

不要把所有验证都堆在最外层，也不要只在最内层测而不做端到端回归。

### 3. 新增行为如果影响成功消息文本，应加 server 侧集成测试

因为这部分直接改变模型能看到什么。对于 agent-native 工具来说，这就是 contract 的一部分。

## 何时考虑新增工具

在考虑新增工具前，先问下面这些问题：

1. 它是否服务于高频主路径？
2. 它能否用现有工具组合得到？
3. 它会不会引入新的提示词负担？
4. 它会不会让工具边界变得模糊？
5. 它是否更适合作为次级接口，而不是核心工具？

如果大部分答案都不清楚，就不应该急着加。

## `apply_splice` 的边界纪律

当维护者处理 `apply_splice` 相关文档或代码时，先区分两类问题：

- 已经关闭的产品边界问题
- 仍待实现收口的工程问题

以下产品边界已经在 `docs/apply_splice_spec.md` 中关闭，不应在普通实现讨论里反复重新打开：

- `apply_splice` 是独立工具，不是 `apply_patch` 的 mode 或表面变体
- `apply_splice` 只转移现有字节，不允许 inline authored text
- `apply_splice` 的当前动作基由八个 transfer 动作加一个 `Delete Span` 原语组成；它不是版本化 scope 菜单，也不应被偷换为 authored replacement 技巧

如果后续有人想改这些点，应把它视为产品边界变更，而不是普通实现细节；必须先更新稳定契约文档，再讨论代码。

相对地，以下问题属于工程闭环，不应反向抛给 PM 做命名式裁决：

- grammar / formal semantics 的稳定落点
- shared mutation substrate 的抽取边界与模块组织
- diagnostics family 的命名、分层与渲染辅助抽象
- parser / selection / runtime / presentation 的实现与测试分层

## 关于 CLI 的维护立场

当前立场是：

- MCP / 注入式 / agent-native 工具接口是主形态
- CLI 如果存在，应优先视为调试、桥接、复现或兼容出口
- 不应让 CLI 语法反过来主导核心 contract

若未来要正式推进 CLI，必须补一份专门设计文档，明确：

- 目标用户是谁
- 与 MCP 的关系是什么
- 哪些语义完全共享
- 哪些只是包装层差异

## 维护者交接建议

新维护者进入本项目时，推荐阅读顺序：

1. `README.md`
2. `docs/product_positioning.md`
3. `docs/maintainer_guide.md`
4. `docs/roadmap.md`
5. 相关专项文档，例如 `docs/apply_patch_semantics_plan.md`

这样可以先建立“项目是什么”的整体心智，再进入具体子系统。
