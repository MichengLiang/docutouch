(deliberation-worklists-index)=
# 70 Worklists

## 作用域

本目录承载从 deliberation 对象派生出的行动组织面。

它回答：接下来为了推进收敛，要做哪些动作。

## 典型对象

- action queue
- next-step list
- review queue
- resolution checklist
- collection task list

## 不承担的职责

- 不定义 issue
- 不定义 proposal
- 不定义 assumption
- 不定义 evidence gap
- 不定义 total execution plan
- 不定义 agent handoff
- 不定义 task matrix
- 不定义 readiness plan
- 不做 migration record

## Dependency Position

### Upstream Dependencies

- `10_issues/`
- `20_proposals/`
- `30_assumptions/`
- `40_candidate_specs/`
- `50_conflicts/`
- `60_evidence_gaps/`

### Downstream Dependents

通常不作为其他家族的 authority 上游。

### Lateral Cross-References

- `15_process_assets/20_work_packages/` 可承接 action-only remainder 之外的结构化执行包
- 可回指 `30_records/status/` 与 `30_records/change/`，记录动作完成后的去向

## Exit Routes

- 动作完成后的事实进入 `30_records/`
- worklist 本身不升格为 `10_knowledge/`

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
