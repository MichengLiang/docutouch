(knowledge-interfaces-pueue-wait-and-log-handle-contract)=
# Pueue Wait And Log Handle Contract

## 作用域

本页记录 DocuTouch 中 Pueue wait surface 与 task-log handle 的 accepted external contract。

它回答：

- `wait_pueue` 的 public parameter shape 与 output grammar 是什么；
- `pueue-log:<id>` handle 在哪些既有工具中合法；
- `read_file` 与 `search_text` 的现有 contract 如何在 handle branch 下保持稳定；
- 调用者应如何理解 wait 与 log inspection 的 interaction loop。

## Interface Position

Pueue integration 在外部 contract 上只承担两件事：

1. `wait_pueue` for real-time task waiting
2. `pueue-log:<id>` handle for existing file/log inspection surfaces

稳定 interaction loop 为：

1. host 通过既有终端能力启动后台命令，例如 `pueue add --print-task-id -- codex exec ...`
2. `wait_pueue` 等待一个或多个 task
3. `read_file` / `search_text` 通过 `pueue-log:<id>` 继续查看日志

本页不把 Pueue integration 写成第二套 CLI toolbox，也不把 metadata helper 暴露成新的 tool family。

## External Contract: `wait_pueue`

### Canonical Signature

```text
wait_pueue(
  task_ids?: integer[],
  mode?: "any" | "all",
  timeout_seconds?: positive number
)
```

### Parameter Semantics

- `task_ids`
  - optional；
  - 缺省时，取调用瞬间当前所有未完成 task 的快照；
  - `0` 是合法 task id；
  - duplicate ids 在 resolution 后去重；
  - 若显式给出顺序，则 `resolved_task_ids` 保持 first-appearance order。
- `mode`
  - optional；
  - 只允许 `any` 或 `all`；
  - 缺省值为 `any`。
- `timeout_seconds`
  - optional；
  - 缺省时读取 runtime default；
  - 显式提供时覆盖 default。

### Snapshot Rule

当 `task_ids` 缺省时：

- resolution target 是调用开始瞬间的 unfinished-task snapshot；
- 等待期间后来新出现的 tasks 不属于本次 wait set；
- 若 snapshot 为空，则立即返回 `nothing_to_wait_for`。

### Reason Taxonomy

accepted `reason` 集合为：

- `task_finished`
- `all_finished`
- `timeout`
- `nothing_to_wait_for`

它们的适用条件分别是：

- `task_finished`
  - `mode = any`，且 timeout 前至少一个 target task 进入 terminal state
- `all_finished`
  - `mode = all`，且 timeout 前全部 target tasks 进入 terminal state
- `timeout`
  - timeout 到达时 external completion condition 仍未成立
- `nothing_to_wait_for`
  - 缺省 snapshot 为空，或显式给定 id 集在 resolution 后为空

### Output Grammar

recommended success surface:

```text
wait_pueue:
reason: task_finished
mode: any
resolved_task_ids: 101, 102
triggered_task_ids: 102
pending_task_ids: 101
waited_seconds: 12.8
current_time: 2026-03-29 23:18:05

[1] task 102
  status: Success
  exit_code: 0
  log_handle: pueue-log:102
```

required header fields:

- `reason`
- `mode`
- `resolved_task_ids`
- `waited_seconds`
- `current_time`

conditional header fields:

- `triggered_task_ids`
- `pending_task_ids`

task block fields:

- `status`
- `exit_code` when available
- `log_handle`

`current_time` 对外只承担本机当前时间的可读 surface：

- format: `YYYY-MM-DD HH:mm:ss`
- 不额外暴露 timezone field

### Error Boundary

`wait_pueue` 的 public invalid-argument family 包括：

- 非法 `mode`
- 非法 `task_ids` shape
- 指定的 task id 不存在
- `timeout_seconds <= 0`

daemon reachability failure 不属于 argument error，应返回 truthfully 指向 Pueue daemon 不可连接的 tool failure。

## External Contract: `pueue-log:<id>`

### Object Identity

`pueue-log:<id>` 是 task-log asset handle literal。

它不是：

- filesystem path；
- Pueue native return payload；
- 新的 tool family；
- 广义虚拟 URI tree。

### Minimal Grammar

```text
pueue-log:<non-negative-integer>
```

examples:

- `pueue-log:0`
- `pueue-log:42`

### Accepted Consumers

该 handle 只在以下输入位点合法：

- `read_file.relative_path`
- `search_text.path`
- `search_text.path[]`

它不应被 `list_directory`、`apply_patch` 或 `apply_splice` 接受。

### Resolution Rule

当输入命中该 grammar 时：

1. 解析 `task_id`
2. 解析 Pueue runtime directory
3. 定位 `<runtime>/task_logs/<id>.log`
4. 将 resolved real path 交给现有 file-reading / grouped-search semantics

### `read_file` Contract Preservation

在 `pueue-log:<id>` branch 下，`read_file` 仍必须遵守
{ref}`knowledge-interfaces-read-file-sampled-view-spec`：

- output 继续是 content-first；
- 不新增 metadata header；
- sampled view、line range、line numbers 与 truncation contract 均保持不变。

也就是说：

```text
read_file(relative_path="pueue-log:42", sample_step=5, sample_lines=2)
```

一旦成功，返回体仍然只是日志内容，而不是额外先打印：

- `resolved task id`;
- `resolved path`;
- `task status`。

这些都不属于 `read_file` 的 contract。

### `search_text` Contract Preservation

在 `pueue-log:<id>` branch 下，`search_text` 仍必须遵守
{ref}`knowledge-interfaces-search-text-ux-contract`：

- `search_text` 仍是 discovery surface，不是 reader；
- grouped-by-file rendering 保持稳定；
- `preview` / `full` 两种 view 语义保持稳定；
- `scope` 头字段可 truthfully 显示为输入 handle，如 `pueue-log:42`。

accepted posture 是：

- input scope 可以是 task-centric handle；
- rendering 仍然是 existing grouped-search contract；
- 不新增第二套 Pueue-only search UX。

### Handle Failure Cases

当 handle 无法被成功解析时，应 truthfully 区分：

- `Task does not exist: <id>`
- `Task log not available: <id>`

不得把两类失败混写成 generic path error。

## CLI Projection

与 MCP 保持一一概念映射的 CLI projection 为：

- `docutouch wait-pueue [TASK_ID ...] [--mode any|all] [--timeout-seconds N]`
- `docutouch read pueue-log:42`
- `docutouch search ERROR pueue-log:42`

CLI projection 是 transport adapter，不重新定义本页 contract。

## Prompt-Facing Guidance

对模型的 prompt-facing teaching 应直接教这组主路径：

- 需要等待后台 task 时使用 `wait_pueue`
- 需要继续查看输出时，把 `log_handle` 直接交给 `read_file` 或 `search_text`
- `pueue-log:<id>` 只表示 task 日志资产，不表示 task metadata
- 需要精确内容时使用 `read_file`
- 需要先做 discovery 时使用 `search_text`

## Non-Goals

本页明确不承担：

- metadata helper tool family；
- generalized task URI namespace；
- session-management API；
- transport-specific implementation debate；
- rollout staging.

## Source Basis

- {ref}`knowledge-architecture-pueue-task-handle-adapter`
- {ref}`knowledge-interfaces-read-file-sampled-view-spec`
- {ref}`knowledge-interfaces-search-text-ux-contract`

