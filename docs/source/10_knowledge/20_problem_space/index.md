(knowledge-problem-space-index)=
# 20 Problem Space

## 作用域

本目录承载 accepted problem model 的正式宿主面。

它回答：

- 问题世界由什么构成
- 有哪些 domain objects 与 givens
- 哪些环境约束已经被 accepted

## 典型对象

- ontology / concept model
- domain entities
- business rules / givens
- accepted environment assumptions
- problem-side context model

## 不承担的职责

- 不写 desired commitments
- 不写 architecture solution
- 不写 candidate assumption
- 不写 interface contract

## Dependency Position

### Upstream Dependencies

- `10_positioning/`

### Downstream Dependents

- `30_principles/`
- `40_requirements/`
- `50_architecture/`

### Lateral Cross-References

- `60_decisions/` 可引用 problem-side givens 作为背景
- `90_reference/` 可汇总概念清单，但不反向定义 problem-space objects

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 family 容器本身的 operation surface
```

当前尚无 first-class 成员；后续若出现稳定成员页，应直接作为当前 family 的 member page 进入本目录，而不是预设学科切片宿主。

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
```
