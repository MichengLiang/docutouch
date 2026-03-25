(records-status-index)=
# 60 Status

## 作用域

本目录承载阶段状态与完成状态记录。

它回答：当前阶段推进到什么状态，哪些 gate 已开合，哪些范围已完成到什么程度。

## 典型对象

- phase status
- completion state
- gate state
- milestone state
- propagation review state

## 不承担的职责

- 不定义 object-level truth
- 不定义 issue / proposal / candidate spec 本体
- 不写 migration 细节本身

## Dependency Position

### Upstream Dependencies

- `20_migration/`
- `30_disposition/`
- `40_change/`
- `50_audit/`

### Downstream Dependents

- `70_coverage/`
- root / overview summaries

### Lateral Cross-References

- `10_inventory/` 可作为状态统计的输入，但不应被 status 反向定义
- `15_process_assets/50_readiness/` 可表达 gate 闭合前的准备面，但不代替 status 自身

## Exit / Refresh Logic

status 通常通过更新与 supersede 延续，而不是升格为 accepted knowledge。

当 accepted revision 已触发 propagation review，
status 应明确表达相关 gate 是否仍未闭合，
而不把这种 review 负荷藏在 change record 里。

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 family 容器本身的 operation surface
* - `process_assets_rollout_status.md`
  - Actual record object
  - 记录 `15_process_assets/` 首轮 rollout 的状态与 gate
* - `apply_patch_contract_repair_wave_20260323.md`
  - Actual record object
  - 记录 2026-03-23 `apply_patch` contract repair wave 的执行状态与 closeout
* - `cli_adapter_implementation_status.md`
  - Actual record object
  - 记录 CLI adapter implementation wave 的完成状态
* - `diagnostics_dx_repair_program_status.md`
  - Actual record object
  - 记录 diagnostics DX repair program 作为协调对象的历史状态
* - `read_file_sampled_view_implementation_status.md`
  - Actual record object
  - 记录 `read_file` sampled-view implementation wave 的完成状态
* - `search_text_implementation_status.md`
  - Actual record object
  - 记录 `search_text` implementation wave 的完成状态
* - `diagnostics_contract_sync_wave_20260323.md`
  - Actual record object
  - 记录 2026-03-23 diagnostics contract sync wave 的状态与 handoff
* - `diagnostics_polish_execution_wave_20260323.md`
  - Actual record object
  - 记录 diagnostics polish execution wave 的执行状态与 closeout
* - `apply_splice_closure_status.md`
  - Actual record object
  - 记录 `apply_splice` closure workspace 的阶段状态与 handoff
* - `apply_splice_implementation_status.md`
  - Actual record object
  - 记录 `apply_splice` 当前实现波次的实际 standing
* - `apply_splice_engineering_hardening_status.md`
  - Actual record object
  - 记录 `apply_splice` engineering hardening wave 的执行状态与 closeout
* - `apply_patch_line_number_assist_rollout_status.md`
  - Actual record object
  - 记录 `apply_patch` line-number-assisted locking rollout 的执行状态与 closeout
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
process_assets_rollout_status
apply_patch_contract_repair_wave_20260323
cli_adapter_implementation_status
diagnostics_dx_repair_program_status
read_file_sampled_view_implementation_status
search_text_implementation_status
diagnostics_contract_sync_wave_20260323
diagnostics_polish_execution_wave_20260323
apply_splice_closure_status
apply_splice_implementation_status
apply_splice_engineering_hardening_status
apply_patch_line_number_assist_rollout_status
```
