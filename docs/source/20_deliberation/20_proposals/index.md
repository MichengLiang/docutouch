(deliberation-proposals-index)=
# 20 Proposals

## 作用域

本目录承载候选立场、候选方案与候选组织方式。

它回答：当前有哪些可推进的候选回答。

## 典型对象

- candidate position
- candidate option
- candidate decomposition
- candidate terminology resolution
- candidate structural direction

## 不承担的职责

- 不写纯 issue
- 不写 accepted decision
- 不写完整 candidate spec
- 不写 generic to-do

## Dependency Position

### Upstream Dependencies

- `10_issues/`
- relevant accepted knowledge family

### Downstream Dependents

- `30_assumptions/`
- `40_candidate_specs/`
- `70_worklists/`

### Lateral Cross-References

- `50_conflicts/` 可标出 proposal 之间的显式冲突
- `60_evidence_gaps/` 可标出 proposal 缺失的担保与证据

## Exit Routes

- 若 proposal 被接纳，其结果进入 `10_knowledge/` 对应家族
- 若 proposal 被放弃，进入 `30_records/disposition/`

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 family 容器本身的 operation surface
* - `apply_splice_technical_investigation.md`
  - Process / Deliberation Object
  - 记录 `apply_splice` 实现边界与 reuse strategy 的当前技术路线提案
* - `line_locked_apply_patch_extension_direction.md`
  - Process / Deliberation Object
  - 记录 line-locked `apply_patch` extension 的当前方向性提案
* - `line_locked_apply_patch_syntax_tradeoff.md`
  - Process / Deliberation Object
  - 记录 line-locked `apply_patch` syntax candidate 的当前 trade-off proposal
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
apply_splice_technical_investigation
line_locked_apply_patch_extension_direction
line_locked_apply_patch_syntax_tradeoff
```
