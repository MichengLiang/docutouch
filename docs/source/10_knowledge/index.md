(knowledge-index)=
# 10 Knowledge

## 作用域

本目录承载当前可被下游依赖的知识对象。

这些对象一旦写入本目录，即意味着：

- 它们已经被接纳；
- 它们可以被后续实现、评审与推理依赖；
- 它们不应继续与未收敛对象混写。

## 二级家族

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

这些子树共同构成 accepted knowledge tree 的二级对象家族。

## 依赖图原则

`10_knowledge/` 的二级家族不仅按职责切分，还应显式说明依赖方向。

### 总体目标

- authoring / authority dependency 尽量保持单向
- 允许 cross-reference，但不把一切关联都升级成强依赖
- `decisions/` 与 `reference/` 不反向统治主体家族

### 总体依赖链

```{list-table}
:header-rows: 1

* - 家族
  - 主要上游
  - 主要下游
* - `positioning/`
  - 无内部上游
  - `problem_space/`、`principles/`、`requirements/`
* - `problem_space/`
  - `positioning/`
  - `principles/`、`requirements/`、`architecture/`
* - `principles/`
  - `positioning/`、`problem_space/`
  - `process/`、`requirements/`、`architecture/`、`decisions/`
* - `process/`
  - `positioning/`、`problem_space/`、`principles/`
  - `requirements/`、`architecture/`、`decisions/`、`operations/`
* - `requirements/`
  - `positioning/`、`problem_space/`、`principles/`、`process/`
  - `architecture/`、`interfaces/`、`operations/`、`decisions/`
* - `architecture/`
  - `positioning/`、`problem_space/`、`principles/`、`process/`、`requirements/`
  - `interfaces/`、`operations/`、`decisions/`
* - `decisions/`
  - `principles/`、`process/`、`requirements/`、`architecture/`、`interfaces/`、`operations/`
  - 一般不作为主体家族的 authority 上游
* - `interfaces/`
  - `principles/`、`process/`、`requirements/`、`architecture/`
  - `operations/`、`decisions/`
* - `operations/`
  - `principles/`、`process/`、`requirements/`、`architecture/`、`interfaces/`
  - 一般位于 accepted knowledge tree 下游
* - `reference/`
  - 各主体家族
  - 一般不作为主体家族的 authority 上游
```

## 页面目录

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向 `10_knowledge/` 容器本身的 operation surface
* - `10_positioning/` 至 `90_reference/`
  - Family container
  - 承载 accepted knowledge tree 的二级 family 容器
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
10_positioning/index
20_problem_space/index
30_principles/index
35_process/index
40_requirements/index
50_architecture/index
60_decisions/index
70_interfaces/index
80_operations/index
90_reference/index
```
