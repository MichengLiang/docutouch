(process-assets-agent-handoff-example)=
# Agent Handoff Minimal Example

## Task Objective

补齐 `15_process_assets/` 与 `70_worklists/`、`60_status/`、`70_coverage/`、`50_audit/` 的边界表述。

## Read These First

- `temporary/docs/source/15_process_assets/index.md`
- `temporary/docs/source/20_deliberation/70_worklists/index.md`
- `temporary/docs/source/30_records/60_status/index.md`
- `temporary/docs/source/30_records/70_coverage/index.md`
- `temporary/docs/source/30_records/50_audit/index.md`

## Allowed Edit Surface

- `temporary/docs/source/20_deliberation/70_worklists/**`
- `temporary/docs/source/30_records/50_audit/**`
- `temporary/docs/source/30_records/60_status/**`
- `temporary/docs/source/30_records/70_coverage/**`

## Disallowed Areas

- `temporary/docs/source/10_knowledge/**`
- `temporary/docs/source/00_meta/30_taxonomy_and_facets.md`
- `temporary/docs/source/15_process_assets/**`

## Exact Deliverable

- 明确 `worklists` 不承接 total execution plan / handoff / matrix / readiness
- 明确 `status` 与 `readiness` 的边界
- 明确 `coverage` 与 matrix / status 的边界
- 明确 `audit` finding 的 follow-up 不吞并 process asset

## Verification Criteria

- 被编辑页均出现对新树的显式边界指认
- 不引入新的顶级 taxonomy 变化
- Sphinx build 通过

## Escalation Conditions

- 若必须改动 `00_meta/` 才能完成，停止并回报
- 若出现 `records` 与 `process_assets` 的宿主冲突，停止并回报

## Report-Back Format

- changed files
- boundary clarified
- remaining ambiguity
