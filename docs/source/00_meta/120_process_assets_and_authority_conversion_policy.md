(meta-process-assets-and-authority-conversion)=
# 过程资产与 Authority 转换政策

## 角色

本页定义外围 planning / stage / matrix / audit / discussion 等过程资产，
如何与当前构建根内的 authority objects、records objects 与其他正式宿主面发生转换关系。

它回答：

- 什么是 process asset；
- 什么是 source material；
- 什么是 source authority；
- 什么是 records authority article；
- 什么是 actual record object。

## 对象类型区分

```{list-table}
:header-rows: 1

* - 对象类型
  - 主要职责
  - 典型位置
* - Process Asset
  - 承担 planning、readiness、matrix、discussion coordination
  - `15_process_assets/**`
* - Source Material
  - 提供 antecedent、证据、历史来源
  - legacy docs、chat extracts、temporary notes
* - Source Authority
  - 承担当前正式边界、规则、正文裁判
  - 构建根内承担正式边界与裁判职责的 authority pages
* - Records Authority Article
  - 定义某类 record 的 scope、boundary、gate、refresh rule
  - `30_records/**/*_scope.md`、`*_rule.md`
* - Actual Record Object
  - 记录某次迁移、处置、变更、审查、覆盖事实
  - future record pages
```

## 构建根与 Authority Role 的区分

`source/` 是当前文档系统的构建根，
回答的是“对象是否进入当前可构建语料”。

`source authority` 则是 authority role，
回答的是“对象是否承担当前正式边界、规则或正文裁判”。

二者不得互相偷换：

- 进入 `source/`，不自动推出对象已经成为 `source authority`；
- 不属于 `source authority`，也不自动推出对象必须位于 `source/` 之外；
- process asset、projection、support surface 与 records object，
  都可以作为 build-root resident object 进入当前可构建语料；
- “进入构建根”与“升格为 authority”是两条不同的状态变化轴。

## 转换规则

### Process Asset -> Build-Root Process Host

若某 planning、handoff、matrix、readiness 或 coordination 对象：

- 需要稳定地址；
- 需要持续维护；
- 需要被目录、搜索、渲染与 cross-reference 系统发现；
- 尚未承担 accepted truth 或 actual record 职责；

则它应进入 `15_process_assets/` 对应子树。

### Process Asset -> Source Authority

只有当以下条件成立时，process asset 中的判断才应升格为 authority-bearing page：

- 该判断已不再只是计划或工作编排；
- 该判断开始长期裁定本地对象分类或边界；
- 该判断已具备本地化重写条件。

若对象只是进入构建根中的 dedicated process host，
而未承担正式边界与裁判职责，
则它仍是 process asset，而不是 source authority。

### Process Asset -> Records Authority Article

若某 process asset 的输出不是具体事实记录，
而是对某类 records 的边界、gate、refresh rule 作出长期裁定，
则它应转化为 records authority article。

### Process Asset -> Actual Record Object

若某 process asset 已在描述某次具体迁移、具体处置、具体审查、具体覆盖结果，
则它不应继续停留在 planning / audit memo 形态，
而应转化为 actual record object。

### Source Material -> Source Authority

source material 只有在经过：

1. 承重断言提取；
2. 口语与上下文依赖移除；
3. 正式术语重写；
4. 层级与 authority role 判定；

之后，才可进入构建根内的适当 authority host。

## 本页与 `60_promotion_and_disposition_policy.md` 的边界

`60_promotion_and_disposition_policy.md` 主要处理：

- `knowledge`
- `deliberation`
- `records`

三域内部的对象迁移。

本页处理的是：

- build root 内外的 planning / stage / matrix / source material
- 如何进入构建根内的 process host、source authority、records authority article 或 actual record object。

二者不应混写。

## 对当前阶段文档的直接含义

当前以下对象首先应被理解为 process assets，而不是 source authority：

- master plan
- stage specs
- readiness plan
- readiness audit
- task matrix

它们可以进入当前构建根并被持续维护，
也可以只作为外围工作底稿存在；
无论位于何处，都不因存在而自动成为 `source authority`。

当前 accepted 路线下，
若它们已经具备 build-root resident 价值，
其 canonical host 即为 `15_process_assets/`。

## 结论

planning、stage、matrix、audit memo 与当前构建根的关系，
不能继续只靠隐含实践维持。

只要当前仓库持续依赖这些对象提供结构判断，
就必须有一页本地 authority 正式说明它们怎样转换、何时转换、转换成什么。
