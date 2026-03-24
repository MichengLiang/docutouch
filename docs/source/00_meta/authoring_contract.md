(meta-contract)=
# 00 Meta 作者契约

## 契约范围

本页规定 `00_meta/` 目录如何继续被扩写、裁剪与维护。

它裁定：

- 什么内容属于元知识正文；
- 什么内容应写入根 `authoring_contract.md`；
- 什么内容应下放到 `10_knowledge/` 的具体知识子树；
- 什么内容若未收敛必须进入 `20_deliberation/`；
- 什么内容若只剩留痕价值必须进入 `30_records/`。

## 可进入本目录的内容

- 关于整个体系的问题定义、适用边界与非目标
- 关于方法论采用与不采用的说明
- 关于树状分类与横切分面的说明
- 关于容器对象、原子对象与 page / folder / section 边界的说明
- 关于 container surface、member surface 与 object kind 的说明
- 关于理论映射精度的说明
- 关于 MyST / Sphinx 表达面选择的说明
- 关于本地 authority 与 external grounding 的边界说明
- 关于外围 process assets 向 dedicated process host、source authority 与 records 的转换说明
- 关于 canonical source、template-role overlay 与 exported template 之间 authority standing 的说明
- 关于对象状态变化规则的说明
- 关于全体系稳定术语的定义

## 不应进入本目录的内容

- 具体项目的 accepted principles 或 decisions
- 某个领域对象的正式知识正文
- 框架自身尚未收敛的争议页
- 迁移、处置、覆盖、done / not done 记录
- 具体模板族的 block inventory、字段表、导出脚本或局部 pipeline 细节
- 只规定操作、不提供法理说明的根级 contract 条文

## 下放规则

### 下放到根 `authoring_contract.md`

若某条内容主要裁定的是“作者必须怎么做”，而不是“体系凭什么这样设计”，它应进入根契约或局部契约，而不应留在 `00_meta/` 正文页。

例如，若某条内容主要规定的是：

- 某容器允许哪些 template-role blocks；
- 导出前要执行哪些 cleanliness checks；
- 哪些改动会破坏 canonical source 与 exported template 的追溯关系；

则它应进入相应 `authoring_contract.md`，
而不应滞留在 `00_meta/` doctrine 页。

### 下放到 `10_knowledge/`

若某条内容已经是项目或领域对象本身的 accepted knowledge，应下放到 `10_knowledge/` 对应子树，而不应继续停留在元知识层。

### 进入 `20_deliberation/`

若框架本身仍有未决对象、未决切分或未决术语，应进入 `20_deliberation/`，并在 `00_meta/` 中只留下明确的路由说明。

若尚未收敛的是某个模板 grammar、block 族、导出机制或 project-specific pipeline，
它同样应停留在 `20_deliberation/` 的 candidate specs 或 process assets 中，
而不是提前进入 `00_meta/`。

### 进入 `30_records/`

若内容主要记录的是框架的迁移、处置、版本演进与完成状态，应进入 `30_records/`。

## 页面对职责的要求

- `framework_scope` 负责范围与问题定义，不代替 contract
- `methodology_commitments` 负责采用声明，不代替 bibliography
- `taxonomy_and_facets` 负责对象分类逻辑，不代替局部目录索引
- `boundary_types_and_container_semantics` 负责 page / folder / section 的边界语义，不代替局部 contract
- `surface_roles_and_object_kinds` 负责 object kind 与 surface role，不代替局部成员声明
- `first_class_object_and_page_folder_criteria` 负责 first-class 判据，不代替单个目录的成员分类表
- `theory_mapping` 负责来源与映射精度，不代替长篇理论综述
- `writing_and_citation` 负责表达面规则，不代替页面正文
- `self_containment_and_external_grounding_policy` 负责本地 authority 与外部 grounding 的边界，不代替 bibliography
- `process_assets_and_authority_conversion_policy` 负责外围过程资产的转换，不代替具体 records 自身
- `canonical_source_and_template_export_policy` 负责 canonical source、overlay、contract 下沉与 exported template 的 standing，不代替具体模板规格或导出实现
- `promotion_and_disposition_policy` 负责状态变化规则说明，不代替 records 自身
- `glossary` 只做定义与边界，不做 essay

## 精度与引用义务

- `00_meta/` 中任何调用学术来源的判断，都应说明取用对象与排除项；
- 不得只贴框架名而不说明本页到底借了什么；
- 理论映射页应显式标注映射精度；
- 若使用局部 bookkeeping 标签，应明确说明它们是本体系的工作性标记，而非共同体术语。

## 对容器与成员的要求

- pair-page rule 只适用于语义容器，不适用于所有语义边界。
- `index.md` 与 `authoring_contract.md` 指向所在容器对象本身，不是普通成员页。
- 每个容器级 `index.md` 除 `toctree` 外，还应显式声明成员 object kinds。
- 原子规则页、边界页、grammar 页即使被大量引用，也不应仅因重要性而被误判为容器。
