(meta-boundary-types-and-container-semantics)=
# 边界类型与容器语义

## 角色

本页定义当前文档系统中三类边界的区别，以及文件夹、页面、section 分别在何种条件下承担何种边界职责。

它回答的不是目录美观问题，而是：

- 哪些对象是容器；
- 哪些对象只是原子页面；
- `index.md` 与 `authoring_contract.md` 为什么不是普通成员页；
- 一个被大量交叉引用的规则页为什么仍然可以只是 page。

## 三类边界

```{list-table}
:header-rows: 1

* - 边界类型
  - 回答的问题
  - 当前系统中的典型实现
* - 物理边界
  - 对象被放在哪里
  - 文件夹、文件、路径
* - 操作边界
  - 什么能进入、什么时候 hand-off、谁维护
  - `authoring_contract.md`
* - 语义边界
  - 哪些成员属于这里、哪些不属于这里
  - `index.md`、正文 article、support surface
```

这三类边界可能重合，也可能不重合。

因此：

- 有文件夹，不自动推出它已经是语义容器；
- 有边界，不自动推出它应升级为文件夹；
- 有强引用价值，不自动推出它承载成员资格。
- 物理上位于同一目录，不自动推出对象已经发生语义混写或结构冲突。

若多个成员页位于同一容器中，判断是否健康，首先看：

- 该容器的 `index.md` 是否已显式声明 `Member Kinds`
- `authoring_contract.md` 是否已说明 admission / hand-off / prohibition 规则
- 每个成员页是否具有清楚的对象职责与 authority role

因此，物理共置本身不是错误判据；未声明成员类型、未声明边界与规则，才是更直接的治理缺口。

## 容器对象与原子对象

### 容器对象

一个对象只有在同时承担以下职责时，才是容器对象：

- 承载成员；
- 规定成员资格边界；
- 规定成员的进入、退出与 hand-off；
- 需要局部阅读路径与局部维护规则。

当前系统中，容器对象的默认实现是：

- 一个文件夹；
- 其 `index.md` 作为容器的 charter / navigation surface；
- 其 `authoring_contract.md` 作为容器的 operation surface。

### 原子对象

若一个对象：

- 自身就是完整可引用对象；
- 不承载成员资格；
- 不需要局部 admission rule；
- 不需要局部 charter；

则它是原子对象。

原子对象的默认实现是单页，而不是目录。

## `index.md` 与 `authoring_contract.md` 的对象身份

`index.md` 与 `authoring_contract.md` 的对象身份，不是“容器里的两个普通成员页”，而是“当前容器对象的两个不同 surface”。

更准确地说：

- `index.md`
  - 回答当前容器是什么、成员是什么、如何读取；
- `authoring_contract.md`
  - 回答当前容器如何继续维护、成员如何进入与迁出。

因此，二者都直接指向所在文件夹这个容器对象本身。

## 文件夹、页面与 section 的分工

```{list-table}
:header-rows: 1

* - 实现形式
  - 自然适用对象
  - 不应被误当成什么
* - 文件夹
  - 容器对象
  - 不应被误当成“重要页面的放大版”
* - 单页
  - 原子对象、原子规则页、原子 support surface
  - 不应因重要性或高引用度被误判为容器
* - section
  - 页面内部的局部结构
  - 不应在没有独立结构收益时被提前提升为 page 或 folder
```

## 被交叉引用的规则页仍然可以是单页

一个规则页、边界页、grammar 页、status rule 页，即使：

- 被多个页面 `{ref}`；
- 被多个对象视为约束依据；
- 在 query、review、change 中具有高 addressability；

它仍然可以只是原子 page。

原因是：

- “被引用”说明它有 authority value；
- “是容器”则要求它承载成员资格。

前者不推出后者。

## 本页与相邻页面的边界

- 本页回答：边界类型、容器语义、page / folder / section 的对象论。
- {ref}`meta-surface-roles-and-object-kinds` 继续回答：对象类型与 surface 角色的区分。
- {ref}`meta-first-class-object-and-page-folder-criteria` 继续回答：page / folder / section 的 first-class 判据。

## 结论

当前系统中：

- folder 不是“长大的 page”；
- folder 是容器对象的默认实现；
- 单页规则对象即使被大量引用，也仍然可以合法地保持为 page；
- `index.md` 与 `authoring_contract.md` 共同构成容器对象的两个 surface，而不是普通成员页。
