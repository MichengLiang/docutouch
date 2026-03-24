(knowledge-requirements-index)=
# 40 Requirements

## 作用域

本目录承载 accepted requirements family 的正式对象。

它回答：系统被明确要求满足什么。

## 典型对象

- stakeholder needs
- system / software requirements
- constraints
- accepted goals
- acceptance conditions
- trace anchors

## 不承担的职责

- 不写 business givens
- 不写 architecture structure
- 不写 accepted decision record
- 不写 candidate requirement 或 candidate spec

## Dependency Position

### Upstream Dependencies

- `10_positioning/`
- `20_problem_space/`
- `30_principles/`

### Downstream Dependents

- `50_architecture/`
- `60_decisions/`
- `70_interfaces/`
- `80_operations/`

### Lateral Cross-References

- `90_reference/` 可收 requirements inventory 或 glossary support，但不定义 requirement 本体

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
