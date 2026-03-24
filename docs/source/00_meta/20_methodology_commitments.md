(meta-methodology)=
# 方法论承诺

## 角色

本页声明本体系明确采用哪些学术与标准资源，以及各自约束哪一层。

本页不做长篇理论综述，而做采用声明与用途边界说明。

## 分类学、图情学与术语学

我们明确采用以下原则：

- 体系分类法在同一层级上应坚持单一划分依据 {cite:p}`ranganathan1967prolegomena,vickery1960faceted`；
- 横切维度不应伪装成同层目录，而应进入分面、矩阵、标签或交叉引用；
- 术语工作应区分对象、概念、定义与名称，不顺着临时词面直接固化目录与对象 {cite:p}`iso7042022`。

这些原则主要约束：

- 顶级对象分区；
- `knowledge/`、`deliberation/`、`records/` 的内部切分；
- glossary 与命名纪律。

分类法开发顺序上，我们额外参照 Nickerson 等人的 taxonomy development 方法，用于约束“先定元特征、再迭代收敛维度”的开发步骤 {cite:p}`nickerson2013`。

## AGM / DP 与认识论启发

我们将 AGM 与 Darwiche--Pearl 迭代信念修正理论作为上位认识论启发 {cite:p}`alchourron1985logic,darwiche1997iterated,sep-belief-revision`。

我们借的是：

- 现行接受态与未被接纳对象的区分；
- 修正触发、修正轨迹与修正规则的区分；
- “当前可依赖知识”与“尚未进入接受态的对象”不能混写。

我们不借的是：

- 用信念修正理论直接给目录命名；
- 把文档对象简化为纯粹命题逻辑对象。

## 需求工程

我们明确采用 Jackson--Zave、Problem Frames、Twin Peaks 与 Pohl 三维的不同职责 {cite:p}`jackson1995deriving,jackson2001problem,nuseibeh2001twinpeaks,pohl2010requirements`。

- Jackson--Zave / Problem Frames：约束问题域、需求、规格与解域边界
- Twin Peaks：约束未收敛对象与现行知识之间的回写关系
- Pohl 三维：作为检查矩阵，而不作为目录法

## 系统工程与标准

我们采用以下标准与 companion guide 作为对象家族与边界的高强度参考：

- ISO/IEC/IEEE 15288:2023 {cite:p}`iso15288_2023`
- ISO/IEC/IEEE 15289:2019 {cite:p}`iso15289_2019`
- ISO/IEC/IEEE 29148:2018 {cite:p}`iso29148_2018`
- SEBoK 对 stakeholder needs、system requirements、architecture definition 的 companion surfaces {cite:p}`sebok-stakeholder-needs,sebok-system-requirements,sebok-architecture-definition`

这些来源主要约束：

- `knowledge/requirements/`
- `knowledge/architecture/`
- `knowledge/interfaces/`
- `records/` 中与 traceability、change、baseline 有关的家族

## 设计理据与论证

我们采用 Toulmin、IBIS 与 QOC 作为 deliberation / decision 相关对象的主要结构参考 {cite:p}`toulmin1958argument,kunz1970ibis,maclean1991qoc`。

其中：

- accepted decision 进入 `knowledge/60_decisions/`
- 尚未收敛的 issue / option / argument 进入 `deliberation/`

## 架构文档与文档工程

我们采用 42010 与 arc42 作为 architecture description 的对象与表达参考 {cite:p}`iso42010_2022,arc42`。

我们采用 Di{\'a}taxis 作为阅读任务分面参考，而不是顶级对象分区依据 {cite:p}`diataxis`。

## 本仓库内部结构启发

我们参考本项目既有宿主实现中采用的“目录导览页 + 局部维护契约页”分离方式，
作为目录级 `index.md + authoring_contract.md` 对页结构的组织启发。

这是一种结构对应，不宣称为外部共同体标准。

## 参考文献

```{bibliography}
:filter: docname in docnames
```
