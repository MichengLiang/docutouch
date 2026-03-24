(records-migration-docs-markdown-ledger)=
# Docs Root Markdown Migration Ledger

## Role

本页记录当前 `docs/` 根部与 `docs/temporary/` 中原始 Markdown / build asset 的对象裁决与目标宿主。

它回答的是：

- 当前源对象主要承担什么职责；
- 它应迁入哪一个 object-domain / family；
- 迁移动作是 `promote`、`process-host`、`record`、`absorb`、`split-then-migrate` 还是 `keep-as-build-asset`；
- 哪些对象不应被原样提升为 accepted knowledge。

2026-03-24 起，`docs/source/` 外的 Markdown 原件统一归档到 `docs/archive2026年3月24日/`。

- 本页中的 `Source Artifact` 继续保留归档前路径，用于表达裁决发生时的源现实；
- 归档后的实际物理位置由 {ref}`records-migration-docs-external-archive-relocation-20260324` 记录；
- 归档不改变本页对 object-domain、family 与 action 的语义裁决。

## Judgment Basis

本页使用以下本地 authority 作为裁决基线：

- {ref}`docs-root-contract`
- {ref}`meta-taxonomy`
- {ref}`meta-boundary-types-and-container-semantics`
- {ref}`meta-surface-roles-and-object-kinds`
- {ref}`meta-writing-and-citation`

裁决顺序固定为：

1. 先判 object-domain；
2. 再判 family；
3. 再判成员 object kind；
4. 最后决定迁移动作。

本页默认不因“物理相邻”而新建容器；
只有现有容器无法承接成员资格时，才应再开新 host。

## Action Legend

```{list-table}
:header-rows: 1

* - Action
  - 含义
* - `promote`
  - 作为稳定 member page 进入 target host
* - `process-host`
  - 作为 live process asset 进入 `15_process_assets/`
* - `deliberation-host`
  - 作为 live unresolved object 进入 `20_deliberation/`
* - `record`
  - 作为历史留痕对象进入 `30_records/`
* - `absorb`
  - 吸收入现有入口页、index 页或其他 canonical host，不保留同级 canonical page
* - `split-then-migrate`
  - 原页混有多个 coequal 职责，需拆分后分别进入对应 host
* - `keep-as-build-asset`
  - 继续作为构建/运行资产存在，不进入 page tree
```

## Root Docs

```{list-table}
:header-rows: 1

* - Source Artifact
  - Current Duty
  - Target Host
  - Action
  - Notes
* - `docs/README.md`
  - docs 入口与导航说明
  - `source/index.md` 与相关 family `index.md`
  - `absorb`
  - 主要承担 projection / routing；不宜原样保留为 canonical knowledge page
* - `docs/product_positioning.md`
  - accepted positioning knowledge
  - `10_knowledge/10_positioning/product_positioning.md`
  - `promote`
  - canonical page 已建立；作为新的 source-bearing article 进入 positioning family
* - `docs/maintainer_guide.md`
  - accepted maintenance / operational knowledge
  - `10_knowledge/80_operations/maintenance_priorities.md` + `10_knowledge/80_operations/upstream_sync_and_compatibility.md` + `10_knowledge/80_operations/testing_and_tool_admission.md`
  - `split-then-migrate`
  - 已按 operations family 拆为三页；纯 authoring-rule 条款仍应回写到相关 `authoring_contract.md`
* - `docs/roadmap.md`
  - product evolution / priority plan
  - `15_process_assets/10_exec_plans/docutouch_roadmap.md`
  - `process-host`
  - canonical process host 已建立；主职责是 planning，不是 accepted knowledge
* - `docs/apply_patch_diagnostics_spec.md`
  - diagnostics subsystem stable design
  - `10_knowledge/50_architecture/apply_patch_diagnostics_spec.md`
  - `promote`
  - canonical page 已建立；当前先以单页 stable design 承接
* - `docs/apply_patch_semantics_plan.md`
  - mixed: stable semantics + decision record + future hardening order
  - `10_knowledge/70_interfaces/apply_patch_semantics.md` + `10_knowledge/60_decisions/apply_patch_warning_first_rationale.md` + `15_process_assets/10_exec_plans/apply_patch_semantics_hardening_plan.md`
  - `split-then-migrate`
  - accepted semantics 与 accepted rationale 已拆出 canonical page；future hardening order 继续由 process asset 承接
* - `docs/apply_splice_spec.md`
  - stable tool contract
  - `10_knowledge/70_interfaces/apply_splice_spec.md`
  - `promote`
  - canonical page 已建立；作为新的 contract / interface page
* - `docs/cli_adapter_spec.md`
  - adapter / transport architecture
  - `10_knowledge/50_architecture/cli_adapter_spec.md`
  - `promote`
  - canonical page 已建立；主职责是 architecture 和 transport boundary
* - `docs/diagnostics_polish_spec.md`
  - long-lived diagnostics doctrine / judgment rubric
  - `10_knowledge/30_principles/diagnostics_polish_spec.md`
  - `promote`
  - canonical page 已建立；更贴近 accepted principles than process plan
* - `docs/read_file_sampled_view_spec.md`
  - stable tool contract
  - `10_knowledge/70_interfaces/read_file_sampled_view_spec.md`
  - `promote`
  - canonical page 已建立；作为 `read_file` contract page
* - `docs/search_text_design.md`
  - superseded exploratory design background
  - `30_records/30_disposition/search_text_design_disposition.md`
  - `record`
  - canonical disposition page 已建立；已被 `search_text_ux_contract.md` 取代，不再承担 live contract
* - `docs/search_text_ux_contract.md`
  - stable tool contract
  - `10_knowledge/70_interfaces/search_text_ux_contract.md`
  - `promote`
  - canonical page 已建立；作为 `search_text` contract page
* - `docs/ux_hardening_plan.md`
  - UX implementation program
  - `15_process_assets/10_exec_plans/ux_hardening_plan.md`
  - `process-host`
  - canonical process host 已建立；作为活的执行计划对象
* - `docs/make.bat`
  - build asset
  - docs build root
  - `keep-as-build-asset`
  - 继续作为构建资产存在，不进入 page tree
```

## Temporary Root Files

```{list-table}
:header-rows: 1

* - Source Artifact
  - Current Duty
  - Target Host
  - Action
  - Notes
* - `docs/temporary/apply_patch_anchor_semantics_investigation.md`
  - investigation / mismatch finding
  - `30_records/50_audit/apply_patch_anchor_semantics_investigation.md`
  - `record`
  - canonical audit record 已建立；当前价值在于审查与留痕，而非现行知识
* - `docs/temporary/apply_patch_confirmed_gaps_20260323.md`
  - confirmed gaps audit
  - `30_records/50_audit/apply_patch_confirmed_gaps_20260323.md`
  - `record`
  - canonical audit record 已建立；记录已确认缺口与修复方向
* - `docs/temporary/apply_patch_contract_repair_execution_plan.md`
  - executed repair-wave program
  - `30_records/60_status/apply_patch_contract_repair_wave_20260323.md`
  - `record`
  - canonical status record 已建立；已执行 closeout，不再是 live process asset
* - `docs/temporary/apply_patch_diagnostics_implementation_plan.md`
  - historical implementation plan
  - `30_records/30_disposition/apply_patch_diagnostics_implementation_plan_disposition.md`
  - `record`
  - canonical disposition page 已建立；文中已自报 superseded / historical
* - `docs/temporary/apply_patch_locking_strategy_blackbox_evaluation_program.md`
  - future evaluation program
  - `15_process_assets/10_exec_plans/apply_patch_locking_strategy_blackbox_evaluation_program.md`
  - `process-host`
  - canonical process host 已建立；主职责是组织未来比较实验
* - `docs/temporary/apply_splice_implementation_plan.md`
  - implementation plan
  - `15_process_assets/10_exec_plans/apply_splice_implementation_plan.md`
  - `process-host`
  - canonical process host 已建立；当前仍属执行对象
* - `docs/temporary/apply_splice_technical_investigation.md`
  - architecture proposal / reuse study
  - `20_deliberation/20_proposals/apply_splice_technical_investigation.md`
  - `deliberation-host`
  - canonical proposal page 已建立；当前仍在承担 open design input 职责
* - `docs/temporary/cli_adapter_implementation_plan.md`
  - completed implementation plan
  - `30_records/60_status/cli_adapter_implementation_status.md`
  - `record`
  - canonical status record 已建立；已完成，不再作为 live plan
* - `docs/temporary/diagnostics_dx_repair_program.md`
  - partially superseded repair program
  - `30_records/60_status/diagnostics_dx_repair_program_status.md`
  - `record`
  - canonical status record 已建立；当前价值主要是记录该波次 program
* - `docs/temporary/line_locked_patch_design_record.md`
  - future design direction
  - `20_deliberation/20_proposals/line_locked_apply_patch_extension_direction.md`
  - `deliberation-host`
  - canonical proposal page 已建立；当前仍属 open design input
* - `docs/temporary/line_locked_patch_syntax_tradeoff_study.md`
  - future trade-off study
  - `20_deliberation/20_proposals/line_locked_apply_patch_syntax_tradeoff.md`
  - `deliberation-host`
  - canonical proposal page 已建立；当前仍属 open design input
* - `docs/temporary/line_number_alignment_rollout_plan.md`
  - rollout plan
  - `15_process_assets/10_exec_plans/line_number_alignment_rollout_plan.md`
  - `process-host`
  - canonical process host 已建立；只要仍待执行，就保持 process asset
* - `docs/temporary/read_file_sampled_view_implementation_plan.md`
  - completed implementation plan
  - `30_records/60_status/read_file_sampled_view_implementation_status.md`
  - `record`
  - canonical status record 已建立；已完成，不再作为 live plan
* - `docs/temporary/search_text_implementation_plan.md`
  - completed implementation plan
  - `30_records/60_status/search_text_implementation_status.md`
  - `record`
  - canonical status record 已建立；已完成，不再作为 live plan
* - ``docs/temporary/为什么 `apply_splice` 与 `apply_patch` 必须分立.md``
  - accepted rationale for a concrete product boundary
  - `10_knowledge/60_decisions/apply_splice_apply_patch_separation_rationale.md`
  - `promote`
  - canonical decision page 已建立；更贴近 accepted rationale package than general principle
* - `docs/temporary/测试报告1.md`
  - black-box comparison report
  - `30_records/50_audit/apply_patch_blackbox_comparison_report_1.md`
  - `record`
  - canonical audit record 已建立；作为测试审查记录保存
```

## Temporary Subdirectories

### `docs/temporary/engineering_quality_wave_20260323/`

```{list-table}
:header-rows: 1

* - Source Artifact
  - Target Host
  - Action
  - Notes
* - `plan.md`
  - `15_process_assets/10_exec_plans/engineering_quality_wave_20260323_plan.md`
  - `process-host`
  - canonical process host 已建立；仍是当前工程质量波次执行计划
```

### `docs/temporary/diagnostics_contract_sync_20260323/`

```{list-table}
:header-rows: 1

* - Source Artifact
  - Target Host
  - Action
  - Notes
* - `README.md`
  - `30_records/60_status/diagnostics_contract_sync_wave_20260323.md`
  - `absorb`
  - 已吸收到 wave-level status record
* - `execution_plan.md`
  - `30_records/60_status/diagnostics_contract_sync_wave_20260323.md`
  - `record`
  - 已历史化并并入 canonical wave-level status record
* - `doc_sync_matrix.md`
  - `30_records/70_coverage/diagnostics_contract_sync_coverage_20260323.md`
  - `record`
  - canonical coverage record 已建立；更贴近 coverage / synced-vs-remaining map
```

### `docs/temporary/diagnostics_polish_execution/`

```{list-table}
:header-rows: 1

* - Source Artifact
  - Target Host
  - Action
  - Notes
* - `README.md`
  - `30_records/60_status/diagnostics_polish_execution_wave_20260323.md`
  - `absorb`
  - 已吸收到 wave-level status record
* - `diagnostics_polish_execution.md`
  - `30_records/60_status/diagnostics_polish_execution_wave_20260323.md`
  - `record`
  - 已并入 canonical wave-level status record
* - `diagnostics_polish_closeout.md`
  - `30_records/60_status/diagnostics_polish_execution_wave_20260323.md`
  - `record`
  - 已并入 canonical wave-level status record
```

### `docs/temporary/apply_splice_closure/`

```{list-table}
:header-rows: 1

* - Source Artifact
  - Current Duty
  - Target Host
  - Action
  - Notes
* - `README.md`
  - closure workspace overview
  - `30_records/60_status/apply_splice_closure_status.md`
  - `absorb`
  - 已吸收到 closure-level status record
* - `formal_semantics_draft.md`
  - candidate formal semantics
  - `20_deliberation/40_candidate_specs/apply_splice_formal_semantics_draft.md`
  - `deliberation-host`
  - canonical candidate-spec page 已建立；当前仍属 draft semantics，不宜提前升格
* - `acceptance_criteria_draft.md`
  - release / QA gate draft
  - `15_process_assets/50_readiness/apply_splice_acceptance_criteria.md`
  - `process-host`
  - canonical readiness host 已建立；主职责是 gate / readiness criteria
* - `architecture_diagnostics_test_draft.md`
  - implementation-facing QA / diagnostics gate draft
  - `15_process_assets/50_readiness/apply_splice_architecture_diagnostics_and_qa_readiness.md`
  - `process-host`
  - canonical readiness host 已建立；更贴 readiness / QA gate
* - `governance_review.md`
  - governance review
  - `30_records/50_audit/apply_splice_governance_review.md`
  - `record`
  - canonical audit record 已建立；review finding
* - `integration_review.md`
  - integration review
  - `30_records/50_audit/apply_splice_integration_review.md`
  - `record`
  - canonical audit record 已建立；review finding
* - `qa_checklist.md`
  - QA gate checklist
  - `15_process_assets/50_readiness/apply_splice_qa_checklist.md`
  - `process-host`
  - canonical readiness host 已建立；readiness / gate object
* - `review_log.md`
  - review rounds log
  - `30_records/50_audit/apply_splice_review_log.md`
  - `record`
  - canonical audit record 已建立；审查留痕
* - `schedule_plan_draft.md`
  - implementation schedule draft
  - `15_process_assets/10_exec_plans/apply_splice_implementation_schedule_plan.md`
  - `process-host`
  - canonical process host 已建立；planning object
* - `stage_summary.md`
  - closure stage summary
  - `30_records/60_status/apply_splice_closure_status.md`
  - `record`
  - 已并入 canonical closure-level status record
```

## Initial Migration Order (Historical)

该顺序记录本页建立时的首轮迁移推进策略。

截至 2026-03-24：

- 首批 stable knowledge / adjacent records 已开始落入 canonical host；
- 其余对象仍按本页表格中的 `Action` 与 `Target Host` 继续收口；
- 外部原件的注意力隔离由 {ref}`records-migration-docs-external-archive-relocation-20260324` 继续承担。

## Follow-up Obligation

本页只裁定当前源对象的 target host 与迁移动作；
真正迁移时，应同步：

- 更新目标容器的 `Member Kinds` 表
- 更新目标容器的 `toctree`
- 为降格 / 吸收对象建立 successor / canonical host 回指
