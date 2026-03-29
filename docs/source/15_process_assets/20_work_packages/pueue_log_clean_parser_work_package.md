(process-assets-work-package-pueue-log-clean-parser)=
# Pueue Log Clean Parser Work Package

## Objective

落地 `pueue-log:<id>` 的 raw->clean parser kernel，
使 control path 与 visible text 在 server 侧得到稳定分离。

## Upstream Plan

- {ref}`process-assets-exec-plan-pueue-log-clean-surface`

## Required Inputs

- {ref}`deliberation-candidate-specs-pueue-log-clean-surface-draft`
- `playground/loommux/spike/简单命令行构建/cli_demo.py`
- `docutouch-server/src/**`

## Deliverables

- server-owned clean parser module
- OSC pre-scan and rewrite logic
- alt-screen session split logic
- plain path / screen path / fallback path

## Dependencies

- crate selection 已被 candidate spec 锁定
- 不得把 raw 重新暴露为 MCP surface

## Owner Type

- agent

## Acceptance

- plain、CR、ANSI、cursor motion、OSC、alt-screen 六类输入都有稳定 clean output
- parser failure 能回退且不引入 prose

## Exit Route

- 结果交给 {ref}`process-assets-work-package-pueue-log-clean-surface-integration`
- completion status 写入 `30_records/60_status/`
