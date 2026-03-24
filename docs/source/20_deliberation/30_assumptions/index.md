(deliberation-assumptions-index)=
# 30 Assumptions

## 作用域

本目录承载桥接性 assumptions。

它回答：当前为了让 deliberation 能继续推进，暂时接受了哪些前提。

## 典型对象

- bridge assumption
- provisional premise
- provisional scope assumption
- provisional environment assumption

## 不承担的职责

- 不写 accepted principle
- 不写已确认 givens
- 不写纯 issue
- 不写 evidence gap 自身

## Dependency Position

### Upstream Dependencies

- `10_issues/`
- `20_proposals/`
- relevant accepted knowledge family

### Downstream Dependents

- `40_candidate_specs/`
- `70_worklists/`

### Lateral Cross-References

- `60_evidence_gaps/` 可标出 assumption 尚缺的 backing
- `50_conflicts/` 可标出 assumption 与其他对象的不兼容

## Exit Routes

- 若 assumption 被确认，迁入 `10_knowledge/` 对应家族
- 若 assumption 被否定或废弃，进入 `30_records/`

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
