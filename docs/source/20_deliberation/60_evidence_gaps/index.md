(deliberation-evidence-gaps-index)=
# 60 Evidence Gaps

## 作用域

本目录承载使对象暂时无法被接纳的证据、担保与定义缺口。

它回答：为了推进接纳或拒绝，我们还缺什么。

## 典型对象

- missing warrant
- missing backing
- missing source
- missing definition boundary
- missing validation condition

## 不承担的职责

- 不写 generic to-do
- 不写 issue 本体
- 不写 proposal 本体
- 不写 assumption 本体

## Dependency Position

### Upstream Dependencies

- `10_issues/`
- `20_proposals/`
- `30_assumptions/`
- `40_candidate_specs/`
- relevant accepted knowledge family

### Downstream Dependents

- `70_worklists/`
- 相关对象的后续收敛

### Lateral Cross-References

- `50_conflicts/` 可回指某个冲突背后的 evidence gap

## Exit Routes

- gap 被补齐后，相关对象继续推进；
- gap 自身若需要留痕，可进入 `30_records/` 对应家族。

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
