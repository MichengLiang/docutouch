(meta-build-root-and-authority-role-distinction)=
# Build Root 与 Authority Role 区分

## 角色

本页定义：

- 什么是 build root；
- 什么是 authority role；
- 为什么二者是正交维度；
- process asset、records object、projection 等对象如何在 build root 中合法存在而不被误判为 authority。

## 基本判断

`source/` 是当前文档系统的构建根。

它回答的是：

- 当前哪些页面进入可构建语料；
- 哪些页面能够被目录、搜索、交叉引用与渲染系统发现；
- 哪些对象在当前仓库中具备稳定物理宿主位。

authority role 回答的则是：

- 当前对象是否承担正式边界；
- 当前对象是否承担正式规则；
- 当前对象是否承担正文裁判或 records 裁判；
- 当前对象在 authority 结构中是 source authority、records authority、projection、support surface、source material 还是 process asset。

因此，build root 与 authority role 不是同一条轴。

## 不允许的偷换

以下推理均被视为结构错误：

- “对象进入 `source/`，所以它已经是 source authority”；
- “对象不是 source authority，所以它不应进入 `source/`”；
- “对象承担过程职责，所以它只能作为工作区外的临时文件存在”；
- “对象可被持续搜索、渲染与维护，所以它必须进入 accepted knowledge tree”。

这些判断把：

- 物理宿主位；
- 构建可见性；
- authority role；
- object-domain state；

混写成了一条轴。

## Build Root Resident Objects

当前 build root 中，至少允许以下对象合法存在而不自动承担 source authority：

```{list-table}
:header-rows: 1

* - 对象
  - 可进入 build root 的理由
  - 不自动获得的东西
* - Process Asset
  - 需要被持续维护、搜索、渲染与 cross-reference
  - 不自动获得 authority role
* - Actual Record Object
  - 需要被持续可发现与追溯
  - 不自动成为现行知识
* - Projection
  - 需要面向特定读者或工作流稳定导出
  - 不自动成为 canonical source
* - Source Material
  - 在特定阶段可能需要进入 build root 以便整理、清洗、摘取
  - 不自动承担正式裁判
```

## 与 First-class 判据的关系

某对象是否进入 build root，
首先取决于它是否值得成为 first-class object。

其典型判据包括：

- 是否需要稳定寻址；
- 是否需要持续维护；
- 是否需要被目录、搜索、交叉引用与渲染系统发现；
- 是否存在明显的 query / validation 收益。

这些判据说明的是：

- 为什么它值得 materialize；

而不是：

- 它是否已经成为 authority。

## 对 Process Assets 的直接含义

若某 planning、handoff、matrix、readiness 或 coordination 对象：

- 需要跨会话持续维护；
- 需要被多人或多 agent 读取；
- 需要在 build root 中可搜索、可浏览、可交叉引用；
- 需要跟随版本演化而保留稳定地址；

则它可以合法进入 build root，
并继续保持 process asset 身份。

它只有在开始长期裁定本地结构分类、正式边界或规则时，
才应转化为 authority-bearing page。

## 对当前仓库的直接含义

当前仓库已经显式承认：

- process asset 是 object kind；
- source authority 是 authority role；
- records object 是另一类对象；
- build root 是当前 `temporary/docs/source/`。

但若不把“build root 与 authority role 是正交维度”写成本地 authority，
后续关于 planning docs、task matrix、execution plan、handoff page 是否应进入 build root 的争论，
就会持续被错误的二选一框架劫持：

- 要么被误排除在 `source/` 外；
- 要么被误升格为 accepted knowledge。

## 与相邻页面的边界

- 本页回答：build root 与 authority role 的区分。
- {ref}`meta-process-assets-and-authority-conversion` 继续回答：process asset、source material、records article 之间如何转换。
- {ref}`meta-surface-roles-and-object-kinds` 继续回答：object kinds 与 surface roles 的区分。
- {ref}`meta-first-class-object-and-page-folder-criteria` 继续回答：对象何时值得被 materialize。

## 结论

build root 是构建与可发现性的物理宿主轴；
authority role 是对象在裁判结构中的职责轴。

二者正交，不得混写。

只要某过程性对象需要稳定可构建、可搜索、可维护，
它就可以合法进入 build root；
而它是否成为 source authority，
应由 authority role 判定，而不是由路径名偷换得出。
