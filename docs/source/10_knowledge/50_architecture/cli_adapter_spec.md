(knowledge-architecture-cli-adapter-spec)=
# CLI Adapter Architecture

## Role

本页承载 DocuTouch CLI adapter 的 accepted architecture。

它回答的是：

- CLI 在产品体系中的对象身份是什么；
- CLI 与 MCP 如何共享同一 semantic core；
- transport-specific differences 的合法边界在哪里；
- parity 应由哪些 architecture obligations 保证。

## Accepted Boundary

本页处理的是 transport architecture，而不是：

- CLI rollout schedule；
- implementation staging；
- temporary migration notes；
- product positioning 正文。

因此，排期、阶段与 closeout 不应写进本页。

## Architectural Thesis

CLI 是 transport adapter，不是第二套 product semantics layer。

这条判断一旦被 accepted，就意味着：

- CLI 不应平行重写 `search_text`、`apply_patch` 等核心行为；
- transport-specific difference 应收缩在 invocation shape 与 anchor choice；
- semantic parity 比 transport flourish 更重要。

## System Placement

当前可复用的 architecture layers 包括：

- `codex-apply-patch`：patch grammar、execution 与 summary generation；在当前仓库中同时是可被披露 divergence 的 internal substrate，而不是 downstream architecture leash
- `docutouch-core`：filesystem primitives 与 shared runtime shaping
- `docutouch-server`：MCP registration、schema、workspace negotiation 与 server-side glue

CLI adapter 的 architecture position 是：

- 复用 shared semantic core；
- 为命令行 transport 提供 invocation adapter；
- 避免在 server 与 CLI 之间形成双份产品语义。

## Core Architecture Commitments

### 1. Semantic Parity First

CLI 与 MCP 之间必须保持同一组核心语义：

- path resolution behavior
- output grammar
- error classification
- warning behavior
- search omission logic
- patch diagnostics and partial-failure accounting

允许差异仅限于 transport necessity，例如：

- invocation syntax
- absence of explicit `set_workspace`
- path display rooted in current CWD

### 2. CWD As Default Anchor

CLI 不需要显式 `set_workspace` 协议步骤，但 path anchoring 语义并未消失。

accepted contract 是：

- process CWD 是 CLI relative path resolution 的 default anchor；
- absolute path 继续合法；
- rendered path 仍应优先选择可读的 compact display。

但 default anchor 不是“无条件绑定当前 cwd”的禁止例外规则。

对于 DocuTouch 自身生成、且以 file-backed source 重新进入 CLI 的 repair artifact，
accepted transport behavior 可以恢复更真实的 execution anchor，前提是：

- source file 的真实路径可被 truthfully 解析；
- 该路径命中 `<workspace>/.docutouch/failed-patches/*.patch`；
- 恢复出的 `<workspace>` 能直接来自该 artifact path 本身，而不是额外的 hidden CLI state。

在该分支中：

- execution anchor 应恢复为 artifact 所属的 workspace root；
- display anchor 应与 execution anchor 保持一致，避免 diagnostics 退化为偶然的 `../..` 相对显示；
- 这条恢复规则只适用于 DocuTouch-owned failed patch artifact，不应自动推广到任意用户提供的 patch file。

因此，accepted architecture 不是“CLI 拥有 workspace 协议”，而是：

- ordinary CLI invocation 继续以 cwd 为 default anchor；
- DocuTouch-owned repair artifact 允许恢复其原 workspace anchor；
- transport-specific convenience 不得改写普通 file-backed patch 的既有 cwd 语义。

### 3. One-to-One Conceptual Mapping

CLI subcommands 应与 MCP tool family 保持一一对应的概念映射：

- `docutouch list`
- `docutouch read`
- `docutouch search`
- `docutouch patch`

这不是要求 CLI 长得像 JSON，而是要求 mental model 不漂移。

### 4. Shared-Layer Extraction Before Surface Growth

若某些 product semantics 仍滞留在 transport-specific layer，应先抽取 shared semantics，再扩大 CLI surface。

当前高优先级 extraction target 包括：

- `search_text` behavior and rendering
- `apply_patch` success/failure presentation
- path display logic that should not drift across transports

### 5. Parity Testing As Architecture Obligation

CLI parity test 不是上线后的 polish，而是 architecture completeness 的组成部分。

若没有 representative parity coverage，CLI adapter 仍处于 architecture-incomplete 状态。

## Command-Surface Boundary

accepted command surface 只承载统一产品面的命令行投影。

因此 CLI 不应：

- 退化为 raw shell toolbox；
- 镜像底层所有 flag 成为 first-class product surface；
- 借 transport convenience 绕开 DocuTouch 的 semantic contract；
- 取代 MCP 成为 LLM host 的 primary integration surface。

## Testing Boundary

testing architecture 至少要覆盖两层：

- parity tests：验证 CLI 与 MCP 在代表性场景上的 contract alignment
- transport-specific tests：验证 CLI invocation、stdin/file patch input、CWD anchoring 等 adapter-specific behavior

其中与 anchor 相关的 representative coverage 至少应包含：

- ordinary file-backed patch 继续绑定调用时 cwd；
- `.docutouch/failed-patches/*.patch` file-backed retry 可恢复 workspace anchor；
- recovered-anchor path 仍保留原 patch file 作为 truthful diagnostics source。

但 transport-specific tests 不得掩盖 semantic drift。

## Downstream Implications

本页作为 architecture object，可被以下对象引用：

- `70_interfaces/`：说明对外 CLI surface 与 MCP mapping
- `80_operations/`：说明维护与验证职责
- `60_decisions/`：记录局部 accepted rationale

## Non-Goals

- 不把 CLI 建成新的语义层；
- 不用 CLI 名义扩张工具边界；
- 不以阶段性实现顺序定义 architecture truth；
- 不把“暂时复用 server 逻辑”误写成长期结构目标。
