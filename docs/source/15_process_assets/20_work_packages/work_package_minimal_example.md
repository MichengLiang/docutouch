(process-assets-work-package-example)=
# Work Package Minimal Example

## Objective

完成 root topology integration，
使 `15_process_assets/` 真正成为 accepted canonical host。

## Upstream Plan

- {ref}`process-assets-exec-plan-example`

## Required Inputs

- `temporary/docs/source/index.md`
- `temporary/docs/source/authoring_contract.md`
- `temporary/docs/source/00_meta/30_taxonomy_and_facets.md`
- `temporary/docs/source/00_meta/120_process_assets_and_authority_conversion_policy.md`

## Deliverables

- 根级 index / contract 接纳 `15_process_assets/`
- `00_meta/` 中 taxonomy / glossary / conversion policy 同步更新
- `15_process_assets/` family 容器与模板页建立

## Dependencies

- build-root 与 authority role 的区分已被本地 authority 化

## Owner Type

- mixed
  - 结构裁定由主 author 完成
  - 页面实例化与 records 留痕可被 agent 辅助完成

## Acceptance

- root toctree 可发现 `15_process_assets/index`
- taxonomy 与 glossary 中术语一致
- Sphinx build 通过

## Exit Route

- 结果写入 `30_records/40_change/`
- rollout 进度写入 `30_records/60_status/`
