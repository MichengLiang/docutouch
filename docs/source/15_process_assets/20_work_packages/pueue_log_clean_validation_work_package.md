(process-assets-work-package-pueue-log-clean-validation)=
# Pueue Log Clean Validation Work Package

## Objective

补齐样本族、回归测试与 parity coverage，
验证 clean surface 在降噪同时不丢失最终可见文本。

## Upstream Plan

- {ref}`process-assets-exec-plan-pueue-log-clean-surface`

## Required Inputs

- {ref}`deliberation-candidate-specs-pueue-log-clean-surface-draft`
- {ref}`process-assets-work-package-pueue-log-clean-parser`
- {ref}`process-assets-work-package-pueue-log-clean-surface-integration`
- `playground/loommux/spike/简单命令行构建/cli_demo.py`
- `docutouch-server/tests/stdio_smoke.rs`
- `docutouch-server/tests/cli_smoke.rs`

## Deliverables

- canonical sample family updates
- equivalence / boundary / cause-effect regression tests
- read/search parity coverage
- records sink for validation closure

## Dependencies

- parser 与 surface integration 已完成
- candidate spec 的验证边界不得在本包内私自扩张

## Owner Type

- agent

## Acceptance

- 样本族覆盖 candidate spec 中列出的控制类别
- `cargo test -p docutouch-server` 通过
- clean surface 与 raw terminal command 的上下层关系可解释

## Exit Route

- 结果回写 {ref}`process-assets-exec-plan-pueue-log-clean-surface`
- completion status 写入 `30_records/60_status/` 与 `30_records/70_coverage/`
