(records-audit-index)=
# 50 Audit

## 作用域

本目录承载审查发现对象。

它回答：在 review、核查、交叉比对中发现了什么问题、风险或不一致性。

## 典型对象

- audit finding
- consistency review finding
- risk observation
- traceability finding
- rule violation finding

## 不承担的职责

- 不做 unresolved issue 正文
- 不做 generic todo
- 不做 migration log

## Dependency Position

### Upstream Dependencies

- accepted knowledge object
- unresolved deliberation object
- records object itself

### Downstream Dependents

- `20_deliberation/` 中的 issue
- `40_change/`
- `30_disposition/`
- `60_status/`

### Lateral Cross-References

- `70_coverage/` 可引用 audit 结果来说明未覆盖或不一致区域
- `15_process_assets/50_readiness/` 可提供 gate 前准备面，但不代替 audit finding 自身

## Exit / Refresh Logic

audit finding 通常不升格为 accepted knowledge；
它会被解决、转化或留痕，而其记录对象保留在本目录。

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 family 容器本身的 operation surface
* - `apply_patch_confirmed_gaps_20260323.md`
  - Actual record object
  - 记录 2026-03-23 时点已经确认的 `apply_patch` contract gaps
* - `apply_patch_anchor_semantics_investigation.md`
  - Actual record object
  - 记录 2026-03-23 对 `apply_patch` `@@` anchor 机制的审查发现
* - `apply_patch_blackbox_comparison_report_1.md`
  - Actual record object
  - 记录内置编辑工具与 MCP `apply_patch` 的首轮黑盒对比发现
* - `apply_splice_governance_review.md`
  - Actual record object
  - 记录 `apply_splice` closure workspace 的 governance review finding
* - `apply_splice_integration_review.md`
  - Actual record object
  - 记录 `apply_splice` closure workspace 的 integration review finding
* - `apply_splice_review_log.md`
  - Actual record object
  - 记录 `apply_splice` closure review rounds 的审查留痕
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
apply_patch_confirmed_gaps_20260323
apply_patch_anchor_semantics_investigation
apply_patch_blackbox_comparison_report_1
apply_splice_governance_review
apply_splice_integration_review
apply_splice_review_log
```
