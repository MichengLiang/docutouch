(knowledge-principles-index)=
# 30 Principles

## 作用域

本目录承载可被下游依赖的高层原则、政策与长期立场。

它解决的是：哪些 accepted doctrine 将持续约束后续多项局部判断。

## 典型对象

- design principles
- policy-like constraints
- long-lived trade-off doctrine
- no-reopen boundaries

## 不承担的职责

- 不写作者规则
- 不写单次局部 decision 记录
- 不写未收敛的原则提案

## Dependency Position

### Upstream Dependencies

- `10_positioning/`
- `20_problem_space/`

### Downstream Dependents

- `40_requirements/`
- `50_architecture/`
- `60_decisions/`
- `70_interfaces/`
- `80_operations/`

### Lateral Cross-References

- `60_decisions/` 可引用 principles 作为 accepted doctrine 依据
- `90_reference/` 可汇总原则词表，但不反向定义 principles

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 family 容器本身的 operation surface
* - `diagnostics_polish_spec.md`
  - Source-bearing article
  - 承载 diagnostics polishing 的 accepted doctrine、judgment rubric 与 no-reopen boundary
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
diagnostics_polish_spec
```
