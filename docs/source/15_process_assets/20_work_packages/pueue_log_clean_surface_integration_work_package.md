(process-assets-work-package-pueue-log-clean-surface-integration)=
# Pueue Log Clean Surface Integration Work Package

## Objective

把 clean parser 接入 `read_file` / `search_text` 的 `pueue-log:<id>` handle branch，
形成 MCP 的默认 clean surface。

## Upstream Plan

- {ref}`process-assets-exec-plan-pueue-log-clean-surface`

## Required Inputs

- {ref}`deliberation-candidate-specs-pueue-log-clean-surface-draft`
- {ref}`process-assets-work-package-pueue-log-clean-parser`
- `docutouch-server/src/tool_service.rs`
- `docutouch-server/src/cli.rs`

## Deliverables

- `read_file` handle branch 的 clean projection 接线
- `search_text` handle branch 的 clean projection 接线
- read/search parity wiring

## Dependencies

- parser kernel 已可输出稳定 clean text
- 普通 filesystem path 语义不得被污染

## Owner Type

- agent

## Acceptance

- `read_file(pueue-log:<id>)` 返回 clean surface
- `search_text(path=pueue-log:<id>)` 搜索 clean surface
- MCP 中不存在 parallel raw surface

## Exit Route

- 结果交给 {ref}`process-assets-work-package-pueue-log-clean-validation`
- completion status 写入 `30_records/60_status/`
