(process-assets-exec-plan-pueue-task-handle-adapter)=
# Pueue Task-Handle Adapter Implementation Plan

## Purpose

把已 accepted 的 Pueue integration 设计拆成可执行、可并发、可 handoff 的 implementation plan，
使后续 subagent rollout 不需要临场重写目标、边界与依赖关系。

## Source Basis

- {ref}`knowledge-architecture-pueue-task-handle-adapter`
- {ref}`knowledge-interfaces-pueue-wait-and-log-handle-contract`
- {ref}`knowledge-operations-pueue-integration-runtime-and-validation`
- `docutouch-server/src/tool_service.rs`
- `docutouch-server/src/cli.rs`
- `docutouch-server/src/server.rs`
- `docutouch-core/src/fs_tools.rs`

## Target Outcome

- `docutouch-server` 拥有 server-side owned Pueue substrate；
- MCP 新增 `wait_pueue` tool；
- CLI 新增 `wait-pueue` projection；
- `read_file` / `search_text` 接受 `pueue-log:<id>` handle；
- runtime assumptions、Windows path behavior、truthful error boundary 与 parity coverage 全部落地；
- subagent brief、matrix 与 kickoff readiness 均具备 canonical host。

## Scope

- `docutouch-server/src/**`
- `docutouch-server/tests/**`
- `docs/source/15_process_assets/**` 中与本事项相关的 process assets
- implementation 后需要同步的 minimal docs / tool surface（已 accepted contract 不重开）

## Non-Goals

- 不重开 `wait_pueue` / `pueue-log:<id>` 的 accepted interface 边界；
- 不扩张 metadata helper tool family；
- 不把 Pueue integration 扩写成 generic process-management suite；
- 不以本轮为契机重构 `docutouch-core` ontology；
- 不处理根仓库中与 `micheng/docutouch` 无关的 dirty worktree。

## Milestones And Duration

| Milestone | Target Outcome | Expected Duration | Entry Condition | Exit Condition |
| --- | --- | --- | --- | --- |
| M1 | Pueue substrate 与 runtime resolution seam 落地 | same-round | accepted docs 与 process assets 已齐 | server-side owned module 可解析 handle、runtime dir 与 task state |
| M2 | `wait_pueue` tool / CLI projection 落地 | same-round | M1 complete | MCP / CLI 都能 truthfully wait，并返回稳定 summary surface |
| M3 | `pueue-log:<id>` 在 `read_file` / `search_text` 生效 | same-round | M1 complete | 既有读/搜主路径接受 handle branch，且 contract 不漂移 |
| M4 | parity / validation / readiness 收口 | same-round | M2 + M3 complete | tests、docs build 与 kickoff gate 全部闭合 |

## Dependency Strategy

- `M1` 必须先完成，因为它提供 runtime dir、log handle、task snapshot 与 waiter substrate；
- `M2` 与 `M3` 在 `M1` 之后可并行；
- `M4` 依赖 `M2` 与 `M3` 同时完成后统一收口；
- 若 `M1` 暴露出 accepted contract 不足，必须停回主代理，不得由子代理自行扩边界。

## Parallelization Plan

- Stream A：runtime substrate
  - 串行起步，先落地 shared substrate 与 resolution seam。
- Stream B：wait surface
  - 依赖 Stream A；若 `tool_service.rs` / `cli.rs` ownership 仍可清晰切分，则可与 Stream C 并行。
- Stream C：log-handle surface
  - 依赖 Stream A；若与 Stream B 共享热点文件无法切出 disjoint write set，则必须串行跟在 Stream B 之后。
- Stream D：validation / parity / readiness
  - 依赖 Stream B 与 Stream C，统一收口。

最高并发上限是 4，
但当前逻辑最健康的执行形状是：

- 先 1 个 executor 做 Stream A；
- 再优先尝试 2 个 executors 并行做 Stream B / C；
- 若 `tool_service.rs` / `cli.rs` ownership 无法切出 disjoint write set，则退回串行的 `1 -> 1`；
- 最后 1 个 executor 或主代理做 Stream D。

## Acceptance Strategy

- `micheng/docutouch` 子仓库中的 process assets、implementation changes 与 tests 形成闭合变更集；
- `cargo test -p docutouch-server` 通过；
- `uv run python -m sphinx -b dummy docs/source docs/_build_dummy` 通过；
- `wait_pueue`、`docutouch wait-pueue`、`read pueue-log:<id>`、`search ... pueue-log:<id>` 的 docs / code / tests 三者一致；
- 外部 tool surface 仍只新增 `wait_pueue`，不新增 metadata helper tools。

## Risk And Replan Triggers

- 若 Pueue runtime resolution 需要改动 `docutouch-core`，停止并回报；
- 若 `wait_pueue` 与 `pueue-log:<id>` 落地后要求重开 accepted contract，停止并回报；
- 若 `tool_service.rs` 与 `cli.rs` 的 edit surface 发生不可避免冲突，应回到主代理重排 ownership；
- 若 Windows path / daemon reachability 的真实行为与 accepted runtime facts 不一致，应先补 records / docs 再继续实现；
- 若某个子代理无法在自己 handoff 的 disjoint write set 内完成任务，应停止并升级为 plan-level re-slice。

## Related Work Packages

- {ref}`process-assets-work-package-pueue-runtime-substrate`
- {ref}`process-assets-work-package-pueue-wait-surface`
- {ref}`process-assets-work-package-pueue-log-handle-surface`
- {ref}`process-assets-work-package-pueue-validation-and-parity`

## Related Records

- `30_records/50_audit/`
- `30_records/60_status/`
- `30_records/70_coverage/`
