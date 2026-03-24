(meta-taxonomy)=
# 分类法与横切分面

## 角色

本页定义：

- 顶级对象分区为何如此切分；
- 哪些维度是主分法；
- 哪些维度是横切分面；
- 哪些维度不得在同一层级混用。

## 顶级对象分区

当前体系在根目录下使用五个顶级子树：

- `00_meta/`
- `10_knowledge/`
- `15_process_assets/`
- `20_deliberation/`
- `30_records/`

其中：

- `00_meta/` 承载体系自身的已接纳元知识；
- `10_knowledge/` 承载当前可依赖知识；
- `15_process_assets/` 承载 build-root resident process assets；
- `20_deliberation/` 承载尚未收敛对象；
- `30_records/` 承载变化记录对象。

## 主分法

根层使用的是对象域分区，不是：

- 文体分区；
- 渲染分区；
- 读者心理状态分区；
- 单纯的时间先后分区。

这意味着根层回答的是“对象属于哪一类”，而不是“页面写得像什么”。

## 横切分面

以下维度被视为横切分面，而不直接作为根层目录依据：

### 1. authority state

例如：accepted、candidate、superseded、audit-only。

### 2. 阅读任务分面

例如：reference、explanation、how-to、tutorial。

### 3. 形式化程度

例如：informal、semi-formal、formal。

### 4. 来源性质

例如：标准、学术文献、项目裁决、观察性事实、局部 bookkeeping 规则。

## 与 object kind、authority role 的区别

本页处理的是：

- object-domain 分区；
- family 分区；
- 横切分面。

本页**不**直接裁定：

- page / folder / section 的容器语义；
- container surface 与 member surface 的区分；
- source authority / projection / support surface 的 authority role；
- page 与 folder 何时值得 first-class。

这些问题分别由：

- {ref}`meta-boundary-types-and-container-semantics`
- {ref}`meta-surface-roles-and-object-kinds`
- {ref}`meta-first-class-object-and-page-folder-criteria`

继续承担。

## `knowledge/` 的二级家族

`knowledge/` 默认继续按语义职责切分为：

- `positioning/`
- `problem_space/`
- `principles/`
- `process/`
- `requirements/`
- `architecture/`
- `decisions/`
- `interfaces/`
- `operations/`
- `reference/`

这里的切分依据是：对象在现行知识面里承担什么职责。

其中：

- `process/` 承载 accepted process architecture、activity model、interaction / grounding obligation 与 co-evolution rule；
- `operations/` 继续承载系统当前如何被运行、维护、恢复与持续操作的 accepted operational knowledge；
- 二者不得互相吞并。

### `principles/` 与 `decisions/` 的 canonical boundary

```{list-table}
:header-rows: 1

* - 判断维度
  - `principles/`
  - `decisions/`
* - 生效持续时间
  - 倾向于长期持续有效
  - 围绕某个具体议题的已接纳裁决
* - 适用范围
  - 可跨多个后续局部判断复用
  - 通常绑定某个具体对象、议题或选择情境
* - 对象形态
  - 更接近 rule-like accepted knowledge
  - 更接近 case-like accepted knowledge
* - 下游复用方式
  - 作为上位约束被多个主体家族依赖
  - 作为被引用的 accepted rationale 或裁决记录
* - 不应被写成什么
  - 不应写成局部 decision log
  - 不应写成长期 doctrine 或总原则
```

该矩阵是 `principles/` 与 `decisions/` 的权威判别面；局部家族页面只负责具体落实，不再各自发明新的判别标准。

## `deliberation/` 的二级家族

`deliberation/` 默认按未收敛对象类型切分为：

- `issues/`
- `proposals/`
- `assumptions/`
- `candidate_specs/`
- `conflicts/`
- `evidence_gaps/`
- `worklists/`

## `process_assets/` 的二级家族

`process_assets/` 默认按过程性执行职责切分为：

- `exec_plans/`
- `work_packages/`
- `handoffs/`
- `matrices/`
- `readiness/`

## `records/` 的二级家族

`records/` 默认按记录职责切分为：

- `inventory/`
- `migration/`
- `disposition/`
- `change/`
- `audit/`
- `status/`
- `coverage/`

## 不允许混用的划分依据

以下做法被视为结构错误：

- 在同一层把 accepted / candidate / audit 与 positioning / requirements / interfaces 并列；
- 在同一层把 accepted knowledge family 与 process-assets family 混写成同一类对象；
- 把 Di{\'a}taxis 的阅读任务分面直接当成根层对象分区；
- 把作者规则对象与领域知识对象混为一棵并列树；
- 把迁移记录长期停放在现行知识子树中。
- 把 object-domain taxonomy 与 object kind taxonomy 混写成同一条轴；
- 把高引用度的原子规则页误判为容器对象。

## 依赖图不是主分法，但必须被显式设计

顶级对象分区与二级对象家族，回答的是“对象属于哪一类”；
依赖图回答的是“这些对象家族在 authoring 与 authority 上如何单向衔接”。

因此，依赖图不应被伪装成另一条目录树，
但每个重要家族都应显式说明自己的依赖位置。

### 依赖关系的三种基本区分

```{list-table}
:header-rows: 1

* - 关系类型
  - 含义
  - 设计目标
* - 上游依赖
  - 当前家族在 authoring / authority 上以前置家族为基础
  - 尽量保持稀疏、单向、低回路
* - 侧向交叉引用
  - 两个家族经常互相指认，但不互为 authority 前提
  - 允许存在，但不升级为强依赖
* - 派生支撑
  - 某家族主要汇总、索引或辅助组织其他家族对象
  - 不反向统治主体家族
```

### accepted knowledge tree 的依赖目标

在 `10_knowledge/` 中，依赖图的目标不是消灭一切复杂性，
而是：

- 让 authoring / authority dependency 尽量单向；
- 把 unavoidable 的复杂关联保留为 cross-reference，而不是升级成强依赖；
- 避免主体家族互相成为彼此的唯一前提；
- 避免 `reference/` 与 `decisions/` 反向统治主体正文家族。

## 反对预设学科分类

当前模板默认在 `object-domain` 与 `family` 两层完成宿主裁决，
不把软件工程、需求工程或其他学科标签预写进目录树。

更具体地说：

- 学科或领域名称可以出现在正文、metadata、矩阵与交叉引用中，用于表达语境；
- 但它们不自动构成目录边界，也不自动生成新的 member host；
- 若某批对象未来确实形成稳定成员簇，仍应先证明其成员资格边界、局部维护性与 query / validation 收益成立；
- 这种细化也应以对象治理收益裁定，而不是以学科归属命名。

因此，横切分面仍应优先通过 metadata、矩阵、交叉引用与固定 section 表达，
而不是把所有语境轴压进目录树。

后续每个二级家族的 `index.md` 应显式说明：

- `Upstream Dependencies`
- `Downstream Dependents`
- `Lateral Cross-References`

而其 `authoring_contract.md` 应显式说明：

- 哪些反向依赖是不健康的；
- 什么时候 cross-reference 不应升级成 dependency。
