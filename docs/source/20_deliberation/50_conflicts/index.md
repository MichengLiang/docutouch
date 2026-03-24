(deliberation-conflicts-index)=
# 50 Conflicts

## 作用域

本目录承载显式冲突对象。

它回答：哪些主张、候选物、前提或 accepted objects 之间已经明确存在不兼容关系。

## 典型对象

- incompatible claims
- proposal conflict
- assumption conflict
- requirement / architecture tension
- explicit trade-off collision

## 不承担的职责

- 不写泛泛 issue
- 不写单纯 evidence gap
- 不写普通 proposal catalog

## Dependency Position

### Upstream Dependencies

- `10_issues/`
- `20_proposals/`
- `40_candidate_specs/`
- relevant accepted knowledge family

### Downstream Dependents

- `20_proposals/` 的修正
- `70_worklists/`
- 最终的 accepted decision

### Lateral Cross-References

- `60_evidence_gaps/` 可标出冲突背后的 evidence gap
- `30_assumptions/` 可作为冲突中的一方被引用

## Exit Routes

- 若冲突被消解，相关结果优先进入 `10_knowledge/` 对应的 accepted family；
- 只有当消解结果本身是一个 accepted case-like judgment 时，相关结果才进入 `10_knowledge/60_decisions/`
- 若冲突被证明是伪冲突或历史噪声，迁入 `30_records/`

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
