(process-assets-exec-plan-pueue-log-clean-surface)=
# Pueue Log Clean Surface Implementation Plan

## Purpose

把 `pueue-log:<id>` 的 clean-surface candidate spec 拆成可执行、可验收、可继续 handoff 的实施计划，
使后续实现不再依赖聊天上下文维持边界。

## Source Basis

- {ref}`deliberation-candidate-specs-pueue-log-clean-surface-draft`
- {ref}`knowledge-interfaces-pueue-wait-and-log-handle-contract`
- {ref}`docs-root-contract`
- `playground/loommux/spike/简单命令行构建/cli_demo.py`
- `docutouch-server/src/tool_service.rs`
- `docutouch-server/src/cli.rs`
- `docutouch-server/tests/stdio_smoke.rs`
- `docutouch-server/tests/cli_smoke.rs`

## Target Outcome

- MCP 下的 `pueue-log:<id>` 默认只暴露 clean surface；
- `read_file` 与 `search_text` 对 task-log handle 消费同一 clean surface；
- clean-surface parser 能处理 plain、`\r`、ANSI、cursor motion、OSC 与 alt-screen；
- temporary 三件套不再承担 authority，`docs/source` 内的 candidate spec 与 process assets 成为当前可维护宿主；
- validation coverage 能证明 clean projection 在不丢失最终可见文本的前提下降噪。

## Scope

- `docutouch-server/src/**`
- `docutouch-server/tests/**`
- `playground/loommux/spike/简单命令行构建/cli_demo.py`
- `docs/source/15_process_assets/**` 中与本事项相关的 process assets
- 与本事项直接相关的最小 docs/source 更新

## Non-Goals

- 不重写 pueue runtime
- 不把 raw surface 加回 MCP schema
- 不引入 PTY 执行前提
- 不修改 `docutouch-core::fs_tools` 的普通文件语义
- 不把 current candidate spec 直接升格为 accepted knowledge

## Milestones And Duration

| Milestone | Target Outcome | Expected Duration | Entry Condition | Exit Condition |
| --- | --- | --- | --- | --- |
| M1 | clean parser 输入边界与样本族锁定 | same-round | candidate spec 已稳定 | sample family、control classes 与 parser choice 在代码/文档上对齐 |
| M2 | clean parser kernel 落地 | same-round | M1 complete | server 侧已有可复用的 raw->clean transform，覆盖 plain/CR/ANSI/cursor/OSC/alt-screen |
| M3 | MCP surface integration 落地 | same-round | M2 complete | `read_file` / `search_text` 在 handle branch 下统一消费 clean surface |
| M4 | validation / parity / records 收口 | same-round | M3 complete | tests 通过，必要 records sink 与 process asset 收尾完成 |

## Dependency Strategy

- 先锁定 parser 输入边界与 sample family，再写 parser kernel；
- parser kernel 必须先于 MCP integration；
- validation 与 records 收口依赖 parser kernel 与 surface integration 同时完成；
- 若实现暴露出 candidate spec 边界不足，应先回收至 candidate spec，不得边写边扩语义。

## Parallelization Plan

- Stream A：parser kernel
  - 负责 OSC pre-scan、segment split、plain path、screen path 与 fallback
- Stream B：surface integration
  - 负责 `read_file` / `search_text` 的 handle branch 接线
  - 依赖 Stream A
- Stream C：validation
  - 负责样本族命令、回归测试、parity coverage
  - 可与 Stream B 后半段并行

当前最健康的执行形状是：

- 先串行完成 Stream A；
- 再并行推进 Stream B / C；
- 最后统一做 records / docs closeout。

## Acceptance Strategy

- `pueue-log:<id>` 的 MCP surface 只输出 clean text
- `search_text` 与 `read_file` 对同一 handle 的 clean 结果一致
- `cli_demo.py` 样本族能复现实验面并支撑回归
- `cargo test -p docutouch-server` 通过
- 文档中 temporary/source/process asset 的 authority 边界保持一致

## Risk And Replan Triggers

- 若 `vt100` 无法承担 candidate spec 所需的 screen extraction，应先回 candidate spec 重新裁定，而不是在实现中临时扩义
- 若 alt-screen block 的 clean surface 需要额外 object kind，停止并回报
- 若 surface integration 迫使 raw 暴露重新进入 MCP，停止并回报
- 若 validation 发现 candidate spec 对“最终可见内容不丢失”的约束不够精确，应先补 spec 再继续

## Related Work Packages

- {ref}`process-assets-work-package-pueue-log-clean-parser`
- {ref}`process-assets-work-package-pueue-log-clean-surface-integration`
- {ref}`process-assets-work-package-pueue-log-clean-validation`

## Related Records

- `30_records/40_change/`
- `30_records/60_status/`
- `30_records/70_coverage/`
