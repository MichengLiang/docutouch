(meta-theory-mapping)=
# 理论映射与精度表

## 角色

本页记录：

- 各理论、标准、框架被取用到哪里；
- 取用的对象或原则是什么；
- 映射精度如何标记；
- 明确排除什么。

## 精度标记

本体系当前使用四种工作性精度标记：

```{list-table}
:header-rows: 1

* - 标记
  - 含义
* - `直接采用`
  - 直接采用外部来源中的对象、原则或边界
* - `结构对应`
  - 不复用原语法或原对象名，但复用其职责结构
* - `过程对应`
  - 不直接拿对象命名目录，而复用其动态关系或状态变化逻辑
* - `局部规则正当化`
  - 该规则是本体系的局部构造，但可以由前述来源提供正当化
```

## 映射表

```{list-table}
:header-rows: 1

* - 来源
  - 取用内容
  - 落点
  - 精度
  - 明确不借什么
* - Ranganathan / Vickery {cite:p}`ranganathan1967prolegomena,vickery1960faceted`
  - 树状切分与横切分面的区分；同层单一划分依据
  - 根层分区与分类页
  - 直接采用
  - 不直接借其具体图书馆分类表
* - ISO 704 {cite:p}`iso7042022`
  - 对象、概念、定义、名称链条；术语不应顺着临时词面固化
  - glossary、命名规则、理论映射页
  - 直接采用
  - 不把 ISO 704 当成目录模板
* - AGM / DP {cite:p}`alchourron1985logic,darwiche1997iterated,sep-belief-revision`
  - 接受态、未被接纳对象、修正轨迹、修正规则的区分
  - knowledge / deliberation / records / contract 的上位认识论约束
  - 过程对应
  - 不直接用 belief revision 术语给目录命名
* - Jackson--Zave / Problem Frames {cite:p}`jackson1995deriving,jackson2001problem`
  - 问题域、需求、规格、解域边界
  - knowledge/problem_space、requirements、architecture
  - 直接采用
  - 不拿它覆盖整个文档系统
* - Twin Peaks {cite:p}`nuseibeh2001twinpeaks`
  - 未收敛对象与现行知识之间的回写关系
  - deliberation 与 knowledge 的流动规则
  - 过程对应
  - 不把它当目录树
* - Pohl 三维 {cite:p}`pohl2010requirements`
  - 完整度、表征、共识的检查坐标
  - 质量检查与评审
  - 直接采用
  - 不作为目录法
* - 15288 / 15289 / 29148 / SEBoK {cite:p}`iso15288_2023,iso15289_2019,iso29148_2018,sebok-stakeholder-needs,sebok-system-requirements,sebok-architecture-definition`
  - requirements、architecture、information item、traceability 的对象边界
  - knowledge 与 records 的对象家族
  - 直接采用 + 局部规则正当化
  - 不直接复制整套标准章节顺序为根目录树
* - 42010 / arc42 {cite:p}`iso42010_2022,arc42`
  - architecture description、concern / viewpoint / view、稳定章节骨架
  - knowledge/architecture
  - 直接采用 + 结构对应
  - 不把 arc42 扩成整个知识系统的总树
* - Toulmin / IBIS / QOC {cite:p}`toulmin1958argument,kunz1970ibis,maclean1991qoc`
  - issue、argument、option、criterion、decision 的对象结构
  - knowledge/decisions 与 deliberation 子树
  - 直接采用
  - 不把 accepted decision 与 pending issue 混在同一页面职责中
* - Di{\'a}taxis {cite:p}`diataxis`
  - 阅读任务分面
  - 写作与引用规则页
  - 直接采用
  - 不当作顶级对象分区
* - 本项目既有宿主实现中的目录对页结构
  - 目录导览页与局部维护契约页的分离
  - index.md + authoring_contract.md 对页设计
  - 结构对应
  - 不把本项目私有命名误写成外部共同体标准
```

补充说明：分类法开发流程的工作性参照，还额外借用了 Nickerson 等人的 taxonomy development 方法，用于约束“先定元特征、再迭代收敛维度”的开发顺序 {cite:p}`nickerson2013`。
