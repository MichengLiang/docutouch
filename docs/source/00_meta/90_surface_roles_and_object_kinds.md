(meta-surface-roles-and-object-kinds)=
# Surface 角色与对象类型

## 角色

本页定义当前文档系统中的主要对象类型与 surface 角色。

它回答：

- 哪些页是容器 surface；
- 哪些页是 source-bearing article；
- 哪些页是 exported support surface；
- 哪些页是 process object、record object、instance object；
- 为什么 authoring rule 与 domain/support rule 不是同类对象。

## 容器 surface 与成员 surface

### 容器 surface

容器 surface 只指向容器对象本身。

当前默认包括：

- `index.md`
- `authoring_contract.md`

### 成员 surface

成员 surface 指向容器中的某个成员对象，而不是容器本身。

例如：

- `framework_stack.md`
- `relation_status_rule.md`
- `coverage_refresh_rule.md`

## 对象类型表

```{list-table}
:header-rows: 1

* - 对象类型
  - 主要职责
  - 典型实现
* - Container Charter
  - 声明容器 jurisdiction、成员分类、阅读路径
  - `index.md`
* - Local Authoring Contract
  - 声明容器的 admission、handoff、prohibition、maintenance rule
  - `authoring_contract.md`
* - Source-bearing Article
  - 承担正式定义、边界、理据、约束与 doctrine
  - `scope_and_boundary.md`、`framework_stack.md`
* - Exported Support Surface
  - 承担 vocabulary、relation、status rule、targeting rule、schema、support boundary
  - `relation_surface_grammar.md`、`status_and_gate_model.md`
* - Process / Deliberation Object
  - 承担 issue、proposal、assumption、evaluation、argument、worklist 等开放或过程性对象
  - `20_deliberation/**`
* - Record Authority Article
  - 裁定某类 record object 的边界与作用
  - `migration_record_scope.md`
* - Actual Record Object
  - 记录某个具体迁移、处置、变更、审查、覆盖事实
  - future records pages
* - Instance Object
  - 承担某个实际项目、实际示例、实际条目的正文
  - future example / mapping pages
* - Projection
  - 为特定读者或工作流重编 source
  - guide / summary surfaces
* - Derived View
  - 由更细粒度 source 机械或半机械生成
  - generated tables / renderings
* - Source Material
  - 提供 antecedent、证据、灵感、历史来源
  - planning docs、legacy notes、chat excerpts
* - Process Asset
  - 承担 deliberation、planning、readiness、matrix、coordination 等过程职责
  - `15_process_assets/**`
```

## 规则对象的二分

系统中的“规则”至少有两类：

### 1. Authoring Rule

回答：

- 什么能进入容器；
- 什么必须 hand-off；
- 什么不应混入。

其自然宿主是 `authoring_contract.md`。

### 2. Domain / Support Rule

回答：

- 某个语义对象的正式 boundary；
- 某类 relation 的表达法；
- 某类 support surface 的 standing；
- 某类 records 的刷新条件。

其自然宿主是普通成员页，而不是 authoring contract。

因此，规则页并不自动等于 authoring contract。

## 局部成员分类义务

每个容器级 `index.md`，除 `toctree` 之外，都应显式声明本目录成员分成哪些 object kinds。

最低要求应至少回答：

- 成员名称；
- object kind；
- authority role；
- 是否为容器 surface 或成员 surface；
- 若为成员 surface，其自然宿主类别是什么。

只给 `toctree` 而不声明成员对象类型，会迫使阅读者凭文件名猜对象身份。

## 本页与相邻页面的边界

- 本页回答：对象类型与 surface 角色。
- {ref}`meta-boundary-types-and-container-semantics` 回答：边界类型与容器语义。
- {ref}`meta-first-class-object-and-page-folder-criteria` 回答：何时 page / folder / section 应 first-class。
- {ref}`meta-self-containment-and-external-grounding` 回答：这些对象何时必须本地化，何时可继续外部 grounding。

## 结论

当前仓库不能再把“index / contract / 其他页面”当作足够的对象分类。

若不把 container surface、source-bearing article、exported support surface、process object、record object、instance object 等正式分开，本地规则就无法独立治理本地仓库。
