(records-disposition-index)=
# 30 Disposition

## 作用域

本目录承载处置记录对象。

它回答：某个对象最后被怎样判了，例如保留、吸收、拆分、替代、废弃或拒绝。

## 典型对象

- disposition note
- rejection record
- superseded record
- absorbed-into record
- no-longer-authoritative record

## 不承担的职责

- 不做 inventory 列表
- 不做 migration 过程说明
- 不做 accepted decision 本体正文

## Dependency Position

### Upstream Dependencies

- `10_inventory/`
- `20_migration/`
- relevant accepted or resolved object

### Downstream Dependents

- `60_status/`
- `70_coverage/`
- successor / current canonical host

### Lateral Cross-References

- `40_change/` 可与 disposition 共同描述版本替代，但不替代 disposition judgment

## Exit / Refresh Logic

disposition 记录本身保留；
它应显式说明：

- 对象被如何判定；
- 是否存在 successor；
- 是否值得从 knowledge 层选择性回指。

若 canonical identity 已退出当前 active authority，
而问题的核心已是 reject、retire、absorb 或 supersede，
应优先由 `disposition/` 承担最终裁定。

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 family 容器本身的 operation surface
* - `process_assets_candidate_artifacts_disposition.md`
  - Actual record object
  - 记录过程资产候选页退场与 absorbed-into 关系
* - `search_text_design_disposition.md`
  - Actual record object
  - 记录 `search_text` exploratory design draft 的 superseded / background-only 处置
* - `apply_patch_diagnostics_implementation_plan_disposition.md`
  - Actual record object
  - 记录历史 `apply_patch` diagnostics implementation plan 的退场与 successor 关系
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
process_assets_candidate_artifacts_disposition
search_text_design_disposition
apply_patch_diagnostics_implementation_plan_disposition
```
