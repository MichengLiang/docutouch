(knowledge-operations-pueue-integration-runtime-and-validation)=
# Pueue Integration Runtime And Validation

## 作用域

本页记录 Pueue integration 的 accepted runtime assumptions、configuration facts 与 validation obligations。

它回答：

- 运行该 integration 时哪些 runtime facts 必须成立；
- 哪些配置入口属于 accepted operational surface；
- 哪些测试与 parity coverage 是 contract-complete 的必要条件。

## Accepted Runtime Facts

### Pueue Baseline

Pueue integration 的 current baseline 假定：

- `pueue` CLI 可调用；
- `pueue` daemon 可连接；
- runtime directory 中存在 `task_logs/` subtree；
- task log file identity 继续采用 `<task_id>.log`。

这些 facts 是 integration 正常成立的 runtime basis，而不是对外新增工具 surface。

### Current Local Baseline

当前设计以本仓库本机已验证的 `pueue 4.0.4` 为现实基线。

这条说明的作用是锁定当前 contract 的现实宿主，而不是把 version pin 写成 public tool field。

## Configuration Surface

accepted operational configuration facts 包括：

- `DOCUTOUCH_PUEUE_BIN`
  - optional explicit override for the `pueue` executable path
- `DOCUTOUCH_PUEUE_RUNTIME_DIR`
  - optional explicit override for runtime directory resolution
- `DOCUTOUCH_PUEUE_TIMEOUT_SECONDS`
  - default timeout used by `wait_pueue` when `timeout_seconds` is omitted

若这些 overrides 未提供，则 integration 应按以下顺序解析 runtime basis：

1. explicit DocuTouch env override
2. native Pueue config / native Pueue environment
3. OS default runtime layout

这条 resolution order 属于 operation knowledge，而不是 interface contract。

## Validation Obligations

依据 {ref}`knowledge-operations-testing-and-tool-admission`，本次变更命中的关键边界包括：

- alias path boundary
- Windows path boundary
- success-message contract boundary
- new-tool admission boundary

因此 implementation 不得只改代码，不补回归。

### Required `docutouch-server` Coverage

至少应覆盖：

- `wait_pueue` single-task completion
- `wait_pueue` multi-task `any`
- `wait_pueue` multi-task `all`
- `wait_pueue` timeout
- `wait_pueue` empty snapshot
- `read_file` with `pueue-log:<id>`
- `search_text` with `pueue-log:<id>`
- truthfully differentiated failures for `task missing` vs `log missing`

### CLI / MCP Parity Coverage

至少应覆盖：

- MCP `wait_pueue` 与 CLI `wait-pueue` 的 semantic parity
- MCP `read_file(pueue-log:<id>)` 与 CLI `read pueue-log:<id>` 的 parity
- MCP `search_text(..., pueue-log:<id>)` 与 CLI `search ... pueue-log:<id>` 的 parity

### Windows Boundary Coverage

至少应覆盖：

- default Windows runtime directory layout
- explicit runtime dir override
- truthfully rendered task-log resolution errors under Windows paths

## Tool-Admission Consequence

Pueue integration 之所以只新增一个 external tool，
是因为 `wait_pueue` 满足 tool-admission 里的高频主路径与不可替代价值要求；
而 task metadata resolver 与 log-path resolver 不满足。

因此 operational maintenance 还应持续保护这条 admission boundary：

- 不因 implementation convenience 再扩出 metadata helper tool family；
- 不因 debug convenience 再扩出第二套 Pueue log-reading tool；
- 若 future proposal 试图突破该边界，应先进入 `20_deliberation/`，而不是直接 landed 到 public surface。

## Non-Goals

本页不承担：

- architecture rationale
- external field-by-field contract
- candidate rollout schedule
- migration ledger

## Source Basis

- {ref}`knowledge-architecture-pueue-task-handle-adapter`
- {ref}`knowledge-interfaces-pueue-wait-and-log-handle-contract`
- {ref}`knowledge-operations-testing-and-tool-admission`

