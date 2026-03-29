(process-assets-work-package-pueue-runtime-substrate)=
# Pueue Runtime Substrate Work Package

## Objective

建立 `docutouch-server` 中 Pueue integration 的 shared substrate，
为后续 `wait_pueue` 与 `pueue-log:<id>` surface 提供共同的 runtime / resolver / waiter seam。

## Upstream Plan

- {ref}`process-assets-exec-plan-pueue-task-handle-adapter`

## Required Inputs

- {ref}`knowledge-architecture-pueue-task-handle-adapter`
- {ref}`knowledge-interfaces-pueue-wait-and-log-handle-contract`
- {ref}`knowledge-operations-pueue-integration-runtime-and-validation`
- `docutouch-server/src/tool_service.rs`
- `docutouch-server/src/cli.rs`
- `docutouch-server/src/server.rs`
- `docutouch-core/src/fs_tools.rs`

## Deliverables

- 一个 server-side owned Pueue substrate module；
- task-log handle parser；
- runtime directory resolver；
- task log path resolver；
- active-task snapshot helper；
- `any` / `all` wait substrate 的内部基础能力。

## Dependencies

- accepted architecture / interface / operations docs 已稳定；
- 不需要改动 `docutouch-core` ontology。

## Owner Type

- agent

## Acceptance

- shared substrate 不要求新增外部 tool surface；
- downstream stream 可以在不重开 accepted contract 的前提下直接复用该 substrate；
- 若需要新增依赖或 env surface，其 naming 与 existing `DOCUTOUCH_*` family 保持一致；
- 子仓库 build / test 不因 substrate 引入而退化。

## Exit Route

- 结果交给 {ref}`process-assets-handoff-pueue-runtime-substrate`
- completion status 写入 `30_records/60_status/`
