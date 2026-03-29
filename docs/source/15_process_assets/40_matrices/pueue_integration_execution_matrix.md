(process-assets-matrix-pueue-integration-execution)=
# Pueue Integration Execution Matrix

本矩阵覆盖 `wait_pueue` / `pueue-log:<id>` integration 的 implementation streams、ownership 与依赖关系。

它不覆盖：

- 根仓库其他子项目；
- accepted contract 本体；
- actual status record。

| Relation Type | Source | Target | Status | Note |
| --- | --- | --- | --- | --- |
| owns | Stream A runtime substrate | `docutouch-server/src/pueue.rs` | planned | shared substrate canonical host |
| depends-on | Stream B wait surface | Stream A runtime substrate | planned | wait 依赖 runtime resolution 与 waiter seam |
| depends-on | Stream C log-handle surface | Stream A runtime substrate | planned | handle resolution 依赖 runtime dir 与 log path resolver |
| owns | Stream B wait surface | `docutouch-server/src/tool_service.rs` | planned | MCP tool registration / execution path |
| owns | Stream B wait surface | `docutouch-server/src/server.rs` | planned | MCP transport projection |
| owns | Stream B wait surface | `docutouch-server/src/cli.rs` | planned | CLI `wait-pueue` projection |
| owns | Stream B wait surface | `docutouch-server/tests/stdio_smoke.rs` | planned | MCP wait regression coverage |
| owns | Stream B wait surface | `docutouch-server/tests/cli_smoke.rs` | planned | CLI wait parity coverage |
| owns | Stream C log-handle surface | `docutouch-server/src/tool_service.rs` | planned | handle branch for `read_file` / `search_text` |
| owns | Stream C log-handle surface | `docutouch-server/src/cli.rs` | planned | CLI read/search handle projection |
| owns | Stream C log-handle surface | `docutouch-server/tests/stdio_smoke.rs` | planned | MCP handle regression coverage |
| owns | Stream C log-handle surface | `docutouch-server/tests/cli_smoke.rs` | planned | CLI handle parity coverage |
| blocked-by | Stream D validation / parity | Stream B wait surface | planned | 需 wait surface 先落地 |
| blocked-by | Stream D validation / parity | Stream C log-handle surface | planned | 需 handle surface 先落地 |
| conflicts-on | Stream B wait surface | `docutouch-server/src/tool_service.rs` | active | 与 Stream C 共享热点；若无法切出 disjoint write set，则转串行 |
| conflicts-on | Stream C log-handle surface | `docutouch-server/src/cli.rs` | active | 与 Stream B 共享热点；若无法切出 disjoint write set，则转串行 |
| owns | Stream D validation / parity | `docs/source/15_process_assets/50_readiness/pueue_subagent_kickoff_readiness.md` | planned | gate closeout host |
| feeds | {ref}`process-assets-work-package-pueue-runtime-substrate` | {ref}`process-assets-handoff-pueue-runtime-substrate` | planned | substrate handoff package |
| feeds | {ref}`process-assets-work-package-pueue-wait-surface` | {ref}`process-assets-handoff-pueue-wait-surface` | planned | wait handoff package |
| feeds | {ref}`process-assets-work-package-pueue-log-handle-surface` | {ref}`process-assets-handoff-pueue-log-handle-surface` | planned | handle handoff package |
| feeds | {ref}`process-assets-work-package-pueue-validation-and-parity` | {ref}`process-assets-handoff-pueue-validation-and-parity` | planned | validation handoff package |

