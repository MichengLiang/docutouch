(deliberation-issues-index)=
# 10 Issues

## 作用域

本目录承载未决问题对象。

它回答：哪一个边界、选择、分类、裁决点当前尚未被决定。

## 典型对象

- unresolved question
- boundary issue
- classification issue
- decision issue
- unresolved acceptance question

## 不承担的职责

- 不写已成型 proposal
- 不写桥接 assumption
- 不写完整 candidate spec
- 不写 task list

## Dependency Position

### Upstream Dependencies

- relevant accepted knowledge family
- 必要时，相关 records 作为历史背景

### Downstream Dependents

- `20_proposals/`
- `50_conflicts/`
- `60_evidence_gaps/`
- `70_worklists/`

### Lateral Cross-References

- `30_assumptions/` 与 `40_candidate_specs/` 可回指其来源 issue

## Exit Routes

- 若 issue 被解决，相关结果优先进入 `10_knowledge/` 对应的 accepted family；
- 只有当“accepted case-like judgment”本身是主要对象时，相关结果才进入 `10_knowledge/60_decisions/`
- 若被证明只是噪声或被正式处置，进入 `30_records/`

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
