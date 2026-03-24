(meta-canonical-source-and-template-export)=
# Canonical Source 与 Template Export 政策

## 角色

本页定义：

- 何种模板策略已经上升为 framework-level 元知识；
- 为什么“单一真源 + template-role block overlay + progressive disclosure to authoring contract + self-hosted template export”不应只停留在某个 candidate spec；
- canonical source、overlay blocks、`authoring_contract.md` 与 exported template 之间的 authority 边界；
- 什么条件下，同一份本地文档既可自举维护，又可导出干净模板。

## Placement Judgment

这条策略进入 `00_meta/`，而不只留在某个候选规格，条件是它已经开始长期裁定以下问题：

1. 哪个对象才是模板体系的 canonical source；
2. overlay blocks 属于 source authority、support layer 还是 process artifact；
3. 模板导出产物应被判定为 canonical source、projection 还是 derived view；
4. 哪些规则应写在 doctrine 页，哪些应继续下沉到 `authoring_contract.md`。

一旦这些判断反复支配 page placement、authority role 与 export boundary，
它们就不再只是某个项目的实现细节，
而是本地文档系统必须显式持有的结构裁判。

相反，以下内容仍不属于本页：

- 某个具体模板族的 block 名称、字段表与 placeholder inventory；
- 某个 exporter、parser、renderer 或 build hook 的实现方案；
- 某次迁移中采用什么中间格式、脚本或目录布局；
- 仍未收敛的模板 grammar、UI 或 pipeline 试探。

这些对象应继续停留在 candidate spec、process asset 或 records 中，
直到它们不再只是局部试验。

## 核心政策

### 1. Single Source of Truth

若同一语义契约既要被作者持续维护，
又要被复用为模板，
则当前体系应优先保留一个 self-hosted canonical source，
而不是并行维护两份互相追赶的正文。

这个 canonical source 可以是：

- 一个 source-bearing article；
- 或一个容器对象中的 `index.md` / `authoring_contract.md` 配对 surface；
- 或一个 page family 中被明确声明为 canonical host 的那一页。

无论实现形态如何，
承重断言、边界、slot semantics 与 export standing
都应首先在 canonical source 中可读、可引、可审。

### 2. Template-role Block Overlay

template-role block overlay 是附着在 canonical source 之上的 authoring-facing support layer。

它可以承担：

- placeholder、slot、fill-in cue 的标记；
- author-only 的模板操作提示；
- 导出时的保留、替换、裁切或清洗信号。

它不应承担：

- 只存在于 overlay、而不在正文中可恢复的唯一正式定义；
- 需要靠外部脚本语境才能理解的核心边界；
- 与 canonical prose 发生竞争真值的第二套规则。

因此，overlay 必须是 additive overlay，
而不是 parallel source。

若移除 overlay 后，页面已无法独立说明对象边界，
则该页面尚未满足 canonical source 要求。

### 3. Progressive Disclosure to `authoring_contract.md`

关于模板策略的 doctrine 应保留在 source-bearing policy 页；
关于作者如何日常维护该策略的操作规则，
则应 progressively disclose 到相应容器的 `authoring_contract.md`。

应下沉到 contract 的内容包括：

- 哪些 overlay blocks 允许进入该容器；
- 哪些 blocks 只是 author-only，不得冒充正式正文；
- 导出前必须满足哪些 cleanliness gate；
- 哪些修改会破坏 canonical source 与 exported template 的可追溯关系。

不应把这些操作性 gate 反向塞回 doctrine 页，
否则 `00_meta/` 会退化成局部工作指令集。

### 4. Self-hosted Template Export

若模板本身需要稳定地址、持续维护与版本治理，
则它可以由当前 build root 内的 canonical source 自举维护，
并从同一语料导出干净模板。

此时 exported template 的对象身份应被理解为：

- `projection`，若它面向特定作者或工作流重编 canonical source；
- `derived view`，若它主要是由 canonical source 机械或半机械生成的清洗产物。

它不应因为“可直接拿去填写”就被误判为新的 canonical source。

若导出模板必须长期人工重写，
并与 canonical source 形成不可追溯的双写关系，
则当前设计不再满足 self-hosted template export，
而应在两者之间重新裁定：

1. overlay 是否表达不足；
2. exported template 是否其实应升格为另一 first-class source object；
3. 当前模板需求是否仍停留在 deliberation / process asset 阶段。

## 对象判定表

```{list-table}
:header-rows: 1

* - 对象
  - 自然 object kind / role
  - 不应被误判为什么
* - canonical template host
  - source-bearing article，或容器对象的 canonical surface
  - 不应被误判为一次性 process note
* - template-role block overlay
  - canonical host 上的 authoring-facing support layer
  - 不应被误判为第二份 source authority
* - exported clean template
  - projection 或 derived view
  - 不应被误判为新的真源
* - 从模板派生出的具体实例
  - instance object、process asset 或 source material，视其 standing 而定
  - 不应自动继承 canonical authority
```

## 与相邻页面的边界

- {ref}`meta-surface-roles-and-object-kinds` 负责一般 object kind 与 surface role；本页只处理 canonical source、overlay 与 exported template 的关系。
- {ref}`meta-self-containment-and-external-grounding` 负责哪些结构判断必须本地化；本页说明模板导出关系一旦长期支配本地结构，就必须进入本地 authority。
- {ref}`meta-build-root-and-authority-role-distinction` 负责 build root 与 authority role 的正交区分；本页进一步说明 exported template 即使进入 build root，也不因此自动成为 canonical authority。
- {ref}`meta-writing-and-citation` 负责语义职责到表达面的映射；本页不规定具体 block 语法，只规定 block overlay 的法理 standing。

## 结论

“单一真源 + template-role block overlay + progressive disclosure to authoring contract + self-hosted template export”
属于 framework-level 元知识，
前提是它已经开始长期裁定 canonical source、overlay standing、contract placement 与 exported artifact standing。

一旦满足这一条件，
它就不应只留在某个项目候选规格里，
因为那会把本地 authority 边界继续外包给局部实现。
