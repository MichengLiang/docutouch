(meta-self-containment-and-external-grounding)=
# Self-contained 与外部 Grounding 政策

## 角色

本页定义：

- 本地 source authority 对外部文档的依赖边界；
- 哪些判断可以只外部引用；
- 哪些判断一旦长期治理本地结构，就必须本地化；
- 当前文档树达到何种条件才可被视为 self-contained。

## 基本区分

### 外部 Grounding

外部文档可以提供：

- 理论来源；
- 上位判据；
- antecedent examples；
- 历史脉络。

### 本地 Authority

本地 authority 必须能够独立回答：

- 当前仓库对象如何分类；
- 当前仓库 page / folder 如何判定；
- 当前仓库 support surface 放置规则；
- 当前仓库 process asset 如何转换。

若这些问题只能回到外部文档才能裁定，
则当前仓库尚未 self-contained。

## 本地化触发条件

以下任一条件成立时，外部判断必须被本地化为本地 authority：

1. 它反复支配本地结构判断；
2. 它长期决定本地对象分类；
3. 没有它，本地容器或成员 placement 无法裁定；
4. 没有它，本地 `index.md` 与 `authoring_contract.md` 无法说明自己的对象边界。

## 可以继续只做外部引用的情况

以下对象可以继续主要作为外部 grounding 存在：

- 学术史综述；
- 详细理论脉络；
- 不直接改变本地结构判断的扩展讨论；
- 只提供来源，不直接承担本地对象分类职责的 background material。

## Self-contained 的最低判据

当前文档树若要被视为 self-contained，至少应能在本地语料中独立回答：

1. 当前 object-domain 与 family 是什么；
2. 容器对象与原子对象如何区分；
3. container surface 与 member surface 如何区分；
4. 本地有哪些 object kinds；
5. page / folder / section 的判据是什么；
6. support surface 如何安置；
7. build root 与 authority role 如何区分；
8. 外围 process asset 如何进入 process host、source authority 或 records；
9. 局部 index 如何公开声明成员类型。

## 对当前仓库的直接含义

当前 `temporary/docs/source` 已经具备：

- object-domain 分区；
- 一般 taxonomy；
- build root 与 authority role 的区分；
- process asset host 与转换规则；
- 容器语义；
- 对象类型；
- page / folder 判据；
- 一般写作纪律；
- 三域迁移逻辑。

当前 corpus-level blind spot 已不再集中在顶层对象论，
而更多转为 lower-level family grammar 与局部 targetting 的细化问题。

## 结论

外部 grounding 可以继续存在，
但本地 authority 不能把支配本地结构的裁判持续外包出去。

只要某判断反复决定本地 page / folder / object placement，
它就应进入本地 `00_meta`。
