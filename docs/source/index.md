(docs-root)=
# 文档系统总入口

## 作用域

本目录是当前文档系统的构建根。

它承载的不是某个单一项目的产品说明书，而是一套可被持续维护、可被迁移复用的工程文档治理体系。

本体系同时处理五类东西：

- 文档系统自身的元知识
- 当前可依赖的知识对象
- 持续维护的过程性执行对象
- 尚未收敛的论证与设计对象
- 迁移、审计、变更等记录对象

## 本目录不是什么

- 不是单一项目的产品首页
- 不是纯粹的教程站点
- 不是临时工作草稿堆放区
- 不是只面向入门读者的解释性文档

## 顶级结构

```{list-table}
:header-rows: 1

* - 顶级对象
  - 主要职责
  - 不承担的职责
* - `00_meta/`
  - 说明本体系为什么这样组织、借了哪些理论、各层边界为何成立
  - 不直接承载具体项目的正式知识正文
* - `10_knowledge/`
  - 承载当前可依赖、可被下游引用的知识对象
  - 不长期保留未收敛对象或迁移记录
* - `15_process_assets/`
  - 承载可构建、可持续维护的过程性执行对象
  - 不冒充 accepted knowledge 或 actual record
* - `20_deliberation/`
  - 承载尚未收敛、不可被下游依赖的论证与设计对象
  - 不冒充现行知识面
* - `30_records/`
  - 承载迁移、处置、审计、变更、覆盖等记录对象
  - 不承载现行知识正文
```

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `00_meta/`
  - Object-domain container
  - 承载体系自身的元知识容器
* - `10_knowledge/`
  - Object-domain container
  - 承载当前可依赖知识的容器
* - `15_process_assets/`
  - Object-domain container
  - 承载过程性执行对象的容器
* - `20_deliberation/`
  - Object-domain container
  - 承载未收敛对象的容器
* - `30_records/`
  - Object-domain container
  - 承载变化记录对象的容器
```

## 进入路径

- 若你第一次进入本体系，先读 {ref}`docs-root-contract`。
- 若你要知道整套体系凭什么这样组织，先读 {ref}`meta-index`。
- 若你要判断 page / folder / section、container surface / member surface 或 support surface placement，先读 {ref}`meta-boundary-types-and-container-semantics`、{ref}`meta-surface-roles-and-object-kinds` 与 {ref}`meta-first-class-object-and-page-folder-criteria`。
- 若你要放置当前可依赖知识，进入 {ref}`knowledge-index`。
- 若你要放置 execution plan、handoff、matrix 或 readiness 等过程对象，进入 {ref}`process-assets-index`。
- 若你要放置尚未收敛的对象，进入 {ref}`deliberation-index`。
- 若你要记录迁移、审计或处置结果，进入 {ref}`records-index`。

## 当前 DocuTouch 语料入口

- 若你要快速进入当前 DocuTouch 产品定位，先读 {ref}`knowledge-positioning-product-positioning`。
- 若你要进入当前维护与测试纪律，先读 {ref}`knowledge-operations-maintenance-priorities`、{ref}`knowledge-operations-upstream-sync-and-compatibility` 与 {ref}`knowledge-operations-testing-and-tool-admission`。
- 若你要进入当前工具 contract，先读 {ref}`knowledge-interfaces-apply-patch-semantics`、{ref}`knowledge-interfaces-read-file-sampled-view-spec`、{ref}`knowledge-interfaces-search-text-ux-contract` 与相关 interface pages。
- 若你要查看当前迁移裁决账本，进入 {ref}`records-migration-docs-markdown-ledger`。

## 阅读顺序

```{toctree}
:maxdepth: 2

authoring_contract
语义分面与本体映射检索谱系
00_meta/index
10_knowledge/index
15_process_assets/index
20_deliberation/index
30_records/index
```

## 引用基线

全体系当前共享一份根级 [references.bib](references.bib)。

在对象体系和治理体系稳定前，优先维持单一 bibliography 基线，而不急于拆分为多份本地 `.bib`。

仔细阅读与理解 [语义分面与本体映射检索谱系](语义分面与本体映射检索谱系.md) 作为微观文档结构的组织依据。
