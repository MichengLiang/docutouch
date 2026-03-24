(knowledge-positioning-index)=
# 10 Positioning

## 作用域

本目录承载 accepted knowledge tree 中最外层的 framing family。

它回答：

- 我们在处理什么问题
- 这个体系面对哪些 stakeholder concerns
- 范围在哪里
- 非目标是什么

## 典型对象

- problem statement
- system / project positioning
- scope
- non-goals
- stakeholder map
- concern summary

## 不承担的职责

- 不定义问题世界本体
- 不承载 requirements 条目
- 不承载 architecture description
- 不承载 accepted decision record

## Dependency Position

### Upstream Dependencies

无 `10_knowledge/` 内部上游依赖。

### Downstream Dependents

- `20_problem_space/`
- `30_principles/`
- `40_requirements/`
- `50_architecture/`
- `70_interfaces/`
- `80_operations/`

### Lateral Cross-References

- `60_decisions/` 可引用 positioning 作为背景，但这不构成 positioning 的上游
- `90_reference/` 可汇总 positioning 相关术语，但不应反向定义 positioning

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 family 容器本身的 operation surface
* - `product_positioning.md`
  - Source-bearing article
  - 记录 DocuTouch 的产品定位、主消费者、主路径与非目标
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
product_positioning
```
