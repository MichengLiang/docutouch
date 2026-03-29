(process-assets-work-package-pueue-log-handle-surface)=
# Pueue Log-Handle Surface Work Package

## Objective

让既有 `read_file` / `search_text` 接受 `pueue-log:<id>` handle，
同时保持现有 content-first / grouped-search contract 不漂移。

## Upstream Plan

- {ref}`process-assets-exec-plan-pueue-task-handle-adapter`

## Required Inputs

- {ref}`knowledge-interfaces-pueue-wait-and-log-handle-contract`
- {ref}`knowledge-interfaces-read-file-sampled-view-spec`
- {ref}`knowledge-interfaces-search-text-ux-contract`
- {ref}`process-assets-work-package-pueue-runtime-substrate`
- `docutouch-server/src/tool_service.rs`
- `docutouch-server/src/cli.rs`
- `docutouch-server/tests/stdio_smoke.rs`
- `docutouch-server/tests/cli_smoke.rs`

## Deliverables

- `read_file.relative_path` 的 `pueue-log:<id>` branch；
- `search_text.path` / `path[]` 的 `pueue-log:<id>` branch；
- truthful differentiation for `task missing` vs `log missing`；
- 不新增 metadata header 的 read contract 回归；
- grouped-search contract 下的 handle-scope regression tests。

## Dependencies

- Pueue substrate 已能解析 runtime dir 与 task log path；
- 不得重写 `read_file` / `search_text` 的 accepted product identity。

## Owner Type

- agent

## Acceptance

- `read_file` 在 handle branch 下仍保持 content-first；
- `search_text` 在 handle branch 下仍保持 grouped discovery surface；
- `scope` 与 file-grouped rendering truthfully 分层；
- 不引入平行的 Pueue log-reading tool。

## Exit Route

- 结果交给 {ref}`process-assets-handoff-pueue-log-handle-surface`
- completion status 写入 `30_records/60_status/`
