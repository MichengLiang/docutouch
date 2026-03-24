(docs-root-contract)=
# 根作者契约

## 契约范围

本页规定整个构建根的对象分区、落位规则与状态变化规则。

它裁定：

- 哪些对象进入 `10_knowledge/`
- 哪些对象进入 `15_process_assets/`
- 哪些对象只能停留在 `20_deliberation/`
- 哪些对象只能作为 `30_records/` 保存
- 根目录与各级目录应提供哪些固定页面

它不承担：

- 理论家谱综述
- 具体项目知识正文
- 具体迁移账本内容

这些内容分别进入 `00_meta/`、`10_knowledge/`、`30_records/`。

## 顶级对象分区

### `00_meta/`

承载关于本体系自身的已接纳元知识。

### `10_knowledge/`

承载当前可被下游依赖的知识对象。

### `15_process_assets/`

承载可构建、可持续维护、但不自动承担 accepted truth 的过程性执行对象。

### `20_deliberation/`

承载尚未收敛、因而不可被下游依赖的对象。

### `30_records/`

承载迁移、处置、审计、变更、覆盖等记录对象。

## 落位规则

### 进入 `10_knowledge/` 的条件

一个对象进入 `10_knowledge/`，至少应满足：

- 它已经被明确接纳为现行知识；
- 下游实现、评审或后续推理可以依赖它；
- 它不再依赖未决争议才能成立；
- 它的主要职责不是记录变化轨迹，而是陈述现行对象。

### 进入 `20_deliberation/` 的条件

一个对象进入 `20_deliberation/`，通常意味着：

- 它仍在等待担保、反驳处理或进一步证据；
- 它不能作为现行事实被下游依赖；
- 它是议题、提案、假设、候选规格、冲突或证据缺口之一。

### 进入 `15_process_assets/` 的条件

一个对象进入 `15_process_assets/`，通常意味着：

- 它承担 planning、handoff、matrix、readiness 或 coordination 职责；
- 它需要稳定地址与持续维护；
- 它值得进入 build root 以获得搜索、渲染与交叉引用收益；
- 它不应被误写成 accepted knowledge 或 actual record。

### 进入 `30_records/` 的条件

一个对象进入 `30_records/`，通常意味着：

- 它主要记录变化、处置、覆盖、迁移或审计结果；
- 它描述的是“如何变到现在”，而不是“当前对象是什么”；
- 它的价值在于可追溯，而不在于承担现行真值。

## 容器对页规则

根目录以下的重要语义容器，默认应至少携带：

- `index.md`
- `authoring_contract.md`

其中：

- `index.md` 承担 charter / navigation surface
- `authoring_contract.md` 承担 local maintenance rule surface

二者不得合并为一个“又导览又立法”的混合页。

但这条规则只适用于容器对象。

- 原子规则页、边界页、grammar 页与其他高价值成员页，
  即使被大量交叉引用，也不应仅因重要性而被误判为容器；
- 若某个语义边界并不承载成员资格，则不应强行为其建立 pair-page 目录；
- 每个容器级 `index.md` 除 `toctree` 外，还应显式声明成员 object kinds，
  不得让阅读者只能凭文件名猜对象身份。

## 状态变化规则

### 升格

若 `20_deliberation/` 中的对象被接纳为可依赖知识，应迁入 `10_knowledge/` 对应子树，并在原位置改写为显式指针、摘要或处置说明。

### 过程对象入树

若 planning、handoff、matrix、readiness 等对象已经具备稳定可维护的 build-root resident 价值，
应进入 `15_process_assets/` 对应子树，
而不应继续作为无 canonical host 的外围工作底稿漂浮存在。

### 降格

若 `10_knowledge/` 中的对象不再成立、被取代或退出现行知识面，应迁出到 `30_records/` 的适当记录家族，并保留到新对象的指向。

### 处置

若对象只剩历史追溯价值，不再承担现行职责，也不再需要继续论证，则只保留在 `30_records/`。

## 术语与引用义务

- 正式术语优先在 [00_meta/70_glossary](00_meta/70_glossary.md) 定义。
- 站内高价值对象优先使用稳定 label 与 `{ref}` 交叉引用。
- 站外来源优先使用根级 `references.bib` 与 `{cite}` 语法，而不是手写裸链接充当论证依据。
- 引用学术框架时，必须说明“借了什么”，不得只贴框架名称。

## 禁止事项

- 不得把现行知识长期停放在 `20_deliberation/`。
- 不得把未收敛对象伪装成 `10_knowledge/` 的正式页面。
- 不得把迁移、审计记录藏入正式知识正文中。
- 不得让 `note`、`warning` 等辅助 surface 成为唯一 authority surface。
- 不得让目录结构仅因渲染便利而决定对象边界。

## 与局部契约的关系

根契约提供 corpus-level 规则。

各级 `authoring_contract.md` 只能在不违背根契约的前提下补充局部规则，不得静默削弱这里的基本边界。
