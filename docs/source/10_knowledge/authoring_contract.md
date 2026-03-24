(knowledge-contract)=
# 10 Knowledge 作者契约

## 契约范围

本页裁定：什么对象可以进入当前可依赖知识面。

## 可进入对象

- 已接纳原则
- 已接纳过程骨架与过程义务
- 已接纳需求
- 已接纳架构说明
- 已接纳接口契约
- 已接纳决策与必要理据
- 已接纳运维与生命周期知识

## 不应进入对象

- 尚未收敛的议题、提案、假设、candidate spec
- 只承担迁移、处置、审计价值的记录对象
- 只规定作者行为的局部维护规则

## 下游依赖原则

进入本目录的对象，应默认被视为可被下游依赖。

若对象尚不具备这种地位，不应提前写入本目录。

## 依赖纪律

- `10_knowledge/` 的二级家族必须显式说明自己的依赖位置；
- 普通 cross-reference 不自动升级成 authority dependency；
- `decisions/` 与 `reference/` 不应反向成为主体家族的唯一 authority 前提；
- `process/` 只承载 accepted process architecture 与 process obligation，不得伪装成运行操作或 records；
- 若某家族需要依赖多个上游，应明确说明各上游分别提供哪一类约束，而不是笼统写成“相关”。
