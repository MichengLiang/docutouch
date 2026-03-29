(knowledge-architecture-pueue-task-handle-adapter)=
# Pueue Task-Handle Adapter

## 作用域

本页承载 DocuTouch 中 Pueue integration 的 accepted architecture。

它回答：

- Pueue integration 在产品体系中的对象身份是什么；
- 为什么它不应成为第二套 product semantics layer；
- 为什么其实现宿主是 `docutouch-server` 而不是 `docutouch-core`；
- CLI 与 MCP 应如何共享同一 semantic core。

## Accepted Boundary

本页处理的是：

- task-handle adapter 的系统位置；
- server-side owned semantic glue；
- transport parity obligations；
- external-vs-internal object separation。

本页不处理：

- external contract 的字段级 shape；
- rollout / staging / implementation schedule；
- candidate alternatives；
- operational test checklist。

这些内容分别进入 `70_interfaces/`、`15_process_assets/`、`20_deliberation/` 与 `80_operations/`。

## Architectural Thesis

Pueue integration 是 DocuTouch 既有 agent-native runtime surface 上的一层 task-handle adapter，
不是第二套 shell toolbox，也不是平行日志系统。

这条判断一旦被 accepted，就意味着：

- 不应新增一组与 `read_file` / `search_text` 平行的 Pueue 日志工具；
- 不应把内部 task metadata resolver 抬升为模型可见的 MCP tool family；
- 不应让 CLI transport convenience 反向定义主产品 contract；
- 不应把 Pueue 自身的 process management vocabulary 扩写成新的 product ontology。

## Product Placement

依据 {ref}`knowledge-positioning-product-positioning`，DocuTouch 的主定位是：

- 面向大模型代码代理的轻量、结构化、低摩擦基础文件工作台；
- 优先优化 agent-native runtime surface，而不是人类 shell toolbox；
- 优先保护高频主路径的语义稳定与低歧义输出。

在该定位下，Pueue integration 只应补足当前主路径真正缺失的 task wait semantics，
而不应扩张出新的“后台任务全家桶产品面”。

## Zero-Gap Identification

Pueue integration 试图补足的零语义缺口是：

- 一个或多个后台 task 的现实时间等待；
- 等待完成后，把可继续阅读的最小资产句柄交还给调用者。

它不试图补足：

- 通用命令执行；
- 文件读取；
- grouped search；
- 交互式 TUI 会话管理；
- 通用 metadata 查询面。

因此，accepted external delta 只应包括：

- 一个新的 `wait_pueue` tool；
- 既有 `read_file` / `search_text` 输入域上的 task-log handle branch。

## System Placement

当前可复用的 architecture layers 包括：

- `docutouch-core`：filesystem primitives 与 shared runtime shaping；
- `docutouch-server`：tool registration、schema、workspace/path glue、CLI/MCP transport adapter。

accepted placement 是：

- Pueue integration 的 owned implementation 位于 `docutouch-server`；
- `docutouch-core` 不引入 Pueue task semantics；
- task-log handle 先在 server-side glue 中解析为真实文件路径，再交给现有 core file primitives。

其理由是：

- Pueue 引入的是 task id、runtime directory、daemon reachability、real-time wait 与 CLI invocation；
- 这些对象属于 server-side semantic adapter，而不是 filesystem primitive；
- 若把它们塞入 core，会把 task system 误写成 filesystem ontology 的一部分。

## Owned Architecture Objects

### External Objects

对外 contract surface 中，新增或扩展的对象只有：

- `wait_pueue`
- `read_file` 的 `pueue-log:<id>` handle branch
- `search_text` 的 `pueue-log:<id>` handle branch

### Internal Objects

内部必须存在、但不应暴露为模型可见工具的对象包括：

- task-log handle parser
- Pueue runtime directory resolver
- task log file resolver
- active task snapshotter
- `any` / `all` waiter substrate
- task terminal-state summarizer

这条 external-vs-internal separation 是 architecture boundary，而不是 implementation convenience。

## Handle Strategy

task-log handle 的 accepted role 是：

- 为 `read_file` / `search_text` 提供一个最小、单义的 task-log asset literal；
- 避免新增平行日志工具；
- 避免要求调用者先走 metadata tool 再回到文件工具主路径。

accepted posture 不是引入一整族虚拟 URI，也不是把 Pueue 原生命令文本伪装成路径。

accepted minimal object 是：

- `pueue-log:<task_id>`

它的对象身份是 asset handle literal，而不是 filesystem path，也不是 Pueue native return shape。

## CLI / MCP Parity Boundary

依据 {ref}`knowledge-architecture-cli-adapter-spec`，CLI 是 transport adapter，不是第二套 product semantics layer。

因此 parity obligation 为：

- MCP `wait_pueue` 与 CLI `wait-pueue` 共享同一 semantic core；
- `docutouch read pueue-log:42` 与 MCP `read_file(relative_path="pueue-log:42")` 共享同一 handle resolution 行为；
- `docutouch search ERROR pueue-log:42` 与 MCP `search_text(path="pueue-log:42")` 共享同一 grouped-search semantics；
- transport-specific differences 仅允许出现在 invocation shape，而不允许改写 public semantics。

## Non-Goals

当前架构明确不做：

- 把 DocuTouch 扩展为通用 process-management suite；
- 新增 `get_pueue_task`、`pueue_log`、`pueue_status_snapshot` 等平行工具族；
- 把 Codex 或其他 CLI 的会话模型写成 DocuTouch 的新 ontology；
- 在 architecture 层引入 rollout staging 或 candidate alternative catalog。

## Downstream Implications

本页作为 architecture object，直接约束：

- {ref}`knowledge-interfaces-pueue-wait-and-log-handle-contract`
- `80_operations/` 中与 runtime assumptions、testing obligations 相关的 accepted knowledge；
- `docutouch-server` 中 future implementation 的 ownership boundary。

## Source Basis

- {ref}`knowledge-positioning-product-positioning`
- {ref}`knowledge-architecture-cli-adapter-spec`
- {ref}`knowledge-operations-testing-and-tool-admission`

