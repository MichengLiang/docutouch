(meta-writing-and-citation)=
# 写作与引用规则

## 角色

本页规定：在本体系中，哪些 MyST / Sphinx 语法是合适的 authority surface，哪些只是辅助 surface，哪些默认不应滥用。

本页不是 MyST 入门教程，而是“语义职责到表达面”的映射规则页。

## 总原则

### 1. 语义职责先于语法选择

先判断对象承担什么职责，再选择语法。

### 2. authority surface 与辅助 surface 分开

正式定义、边界、约束与理据，必须落在正文、表格、公式或正式 section 中；不得只存在于 `note`、`warning`、`seealso` 等辅助 surface。

### 3. 内容与呈现分离

语法选择服从对象边界，不允许因为视觉便利反向决定对象如何拆分。

### 4. 语法不决定 object kind

一个页面是容器 surface、成员 surface、source-bearing article 还是 exported support surface，
由对象类型与 authority role 决定，而不是由它使用了什么 MyST 语法决定。

### 5. 读者模型

当前默认读者是高强度专业读者，而非入门读者。页面默认采用 scholarly register，不使用教学型铺垫与聊天型过渡语句。

### 6. 对象语言先于作者自述

正式页面应优先使用对象语言，而不是作者对自己写作动作的自述语言。

应优先回答：

- 对象是什么
- 不是什么
- 依据来自哪里
- 这句承担什么逻辑职责
- 为什么必须写这句

不应把以下写作姿态当成知识陈述：

- “我来压一压”
- “我来稳一稳”
- “我来收一下”
- “我来补一层”
- 其他主要描述作者心理动作、写作手势或自我安抚的过程性说法

这类措辞的问题不在于口语化，而在于它们通常不直接承担对象定义、论域限定、来源声明、warrant、边界说明或反驳处理等知识职责。

若必须使用“压缩”“最小”“稳定”“冗余”等词，必须显式说明：

- 压缩了什么、保留了什么、丢弃了什么
- “最小”相对于什么约束成立
- “稳定”相对于什么风险、什么判据成立
- “冗余”承担的是哪一种多通道编码职责，而不是单纯重复

否则应改写为直接的对象与逻辑职责陈述。

## 推荐语法

### `toctree`

主要用于目录级 `index.md` 的稳定导航，不代替正文结构。

### label + `{ref}`

作为站内交叉引用的第一优先级机制。高价值页面与 section 应提供稳定 label。

### `{cite:p}` / `{cite:t}` + `bibliography`

作为站外来源的统一引用机制。引用必须说明取用内容，不得只贴框架名。

### `glossary`

用于稳定术语定义。glossary 条目负责定义与边界，不负责长篇论证。

### `list-table`

用于矩阵、映射表、对象职责表、规则表。若对象仍需长篇边界说明，应先 prose 后表格。

### `math`

仅在 formal relation 是页面核心对象时使用，例如 Jackson--Zave。

### `mermaid`

仅在对象关系、过程流或 hand-off 路径需要图形化时使用。图不能替代正文中的定义与约束。

### `contents`

用于长页面的局部导航，尤其适用于 `00_meta/` 中的高密度页面。

## admonition 语义等级

```{list-table}
:header-rows: 1

* - 语法
  - 主要职责
  - 禁止承担的职责
* - `important`
  - 下游会依赖的关键边界或 invariant
  - 唯一正文定义
* - `warning`
  - 高概率会犯的结构性错误
  - 唯一规则正文
* - `note`
  - 说明性补充与限定
  - 唯一 authority 信息
* - `seealso`
  - 路由与 hand-off
  - 代替对象定义
```

## 慎用与默认不用的语法

- 过强视觉组件：容易让页面观感压倒对象职责
- `dropdown`：容易隐藏应当直面的 authority 内容
- `tabs`：容易切断并列对象的同步可见性
- 过多 footnotes：容易打断高密度正文
- 复杂 substitution / templating：当前阶段优先稳对象边界，而不是写作自动化

## 页面微观骨架

### `index.md`

- `Role`
- `Boundary`
- `What Lives Here`
- `What Does Not Live Here`
- `Member Kinds`
- `Reading Paths`
- `toctree`

### `authoring_contract.md`

- `Contract Scope`
- `Allowed Objects`
- `Disallowed Objects`
- `Placement Rules`
- `State Transition Rules`
- `Cross-Reference Obligations`
- `Failure Modes`

`index.md` 与 `authoring_contract.md` 的职责，只适用于容器对象。
原子规则页、边界页与 support surface 不应因重要性而被强行改写为 pair-page 容器。

### 理论映射页

- `Role`
- `Precision Legend`
- `Mapping Table`
- `Non-Adopted Lines`

## 站内与站外引用分工

- 站内对象：优先 label + `{ref}`
- 站外来源：优先 `{cite}` + `references.bib`

站内路由不得伪装成学术引用，学术引用也不得代替站内对象寻址。

## 交叉引用不等于依赖

页面之间可以互相 `{ref}`，但这并不自动表示它们构成 authoring / authority dependency。

若某目录或页面需要表达依赖方向，应在正文中显式使用：

- `Upstream Dependencies`
- `Downstream Dependents`
- `Lateral Cross-References`

而不是把所有 `{ref}` 都误读为强依赖。
