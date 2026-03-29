(deliberation-candidate-specs-pueue-log-clean-surface-draft)=
# Pueue Log Clean Surface Draft

## 作用域

本页记录 `pueue-log:<id>` 在 DocuTouch MCP 包装层上的 clean-surface candidate spec。

它回答的是：

- `pueue-log:<id>` 在 MCP 中应暴露什么 surface；
- clean surface 的对象边界是什么；
- clean surface 应保留什么、去除什么；
- clean surface 的算法职责、crate 选择与 ownership boundary 是什么；
- 当前仍停留在 deliberation、尚未进入 accepted knowledge 的边界在哪里。

## Target Accepted Families

- `10_knowledge/70_interfaces/`
- `10_knowledge/50_architecture/`
- `10_knowledge/60_decisions/`
- `15_process_assets/10_exec_plans/`

## Source Basis

- `micheng/docutouch/temporary/log-normalizer-specs/PRD.md`
- `micheng/docutouch/temporary/log-normalizer-specs/SRS.md`
- `micheng/docutouch/temporary/log-normalizer-specs/TEST-SPEC.md`
- `playground/loommux/spike/简单命令行构建/cli_demo.py`
- {ref}`knowledge-interfaces-pueue-wait-and-log-handle-contract`
- {ref}`docs-root-contract`

其中：

- `temporary/log-normalizer-specs/*` 仅作为 source material / projection 参考；
- 本页才是当前 deliberation 阶段的 canonical host；
- 任何后续 accepted promotion 都应从本页拆出对应主体家族，而不是把 temporary 三件套直接升格。

## Candidate Object

当前候选对象不是 generic terminal logger，
也不是 pueue runtime 重写方案，
而是：

> `pueue-log:<id>` 在 MCP 中面向 LLM 的 clean text projection。

它的产品立场已经锁定为：

- MCP 默认且只暴露 clean surface；
- raw surface 不进入 MCP tool schema；
- 若模型或人类需要 raw log，直接调用原始终端命令；
- clean 的职责是 terminal-noise elimination 与 visible-text reconstruction；
- clean 不做摘要，不做解释性 prose，不把 screen 内容二次改写成自然语言。

## Candidate Problem Statement

当前 `pueue` integration 暴露的 task-log substrate 会保留：

- plain line text
- `\r`
- ANSI SGR
- cursor motion / clear-line
- OSC
- alt-screen enter / exit

因此当前问题不是“原始信息丢失后如何猜回去”，而是：

- 这些 raw terminal-ish bytes 如何在 MCP 中投影成更低噪声的文字 surface；
- 哪些控制路径应被视为时间冗余；
- 哪些字符结果本身应被保留为正文。

本 candidate spec 明确锁定以下判断：

- 噪声首先来自时间维度的重复重绘；
- 噪声不等于 box-drawing / panel / table / tree / unicode symbol；
- virtual screen 上最终可见的字符内容本身不是噪声；
- MCP 的首要职责是给 LLM 最干净的文字，而不是给 LLM 最原始的控制字节流。

## Candidate Contract Surface

### MCP Surface

- `read_file(relative_path="pueue-log:<id>")` 返回 clean surface
- `search_text(path="pueue-log:<id>")` 搜索 clean surface

### Explicit Non-Surface

- 不新增 `raw` 参数
- 不新增 `surface` 参数
- 不新增 parallel 的 raw log tool
- raw log 不经由 MCP 重新包装

### Raw Escape Hatch

- raw access 通过原始终端命令完成，例如 `pueue log --full --json`
- 这条 escape hatch 不进入 MCP authority surface

## Candidate Locked Elements

当前 draft 已锁定的 candidate-level 事实如下。

### 1. Ownership Boundary

本能力属于 `docutouch-server` 的 Pueue handle branch，不进入 `docutouch-core::fs_tools`。

理由：

- 普通 filesystem path 不应被隐式施加 terminal-cleaning 语义；
- `pueue-log:<id>` 本来就是 server-owned virtual surface；
- clean-vs-raw 的产品决策只属于 task-log domain，不应污染普通文件读取语义。

### 2. Crate Selection

采用：

- `vt100`
  - 用于 terminal parsing + virtual screen representation
- `unicode-width`
  - 用于字符列宽语义

明确不采用：

- `portable-pty`
  - 当前对象不是 runtime 重写，而是既有 task-log substrate 的 clean projection
- 直接以 `vte` 作为一线入口
  - 当前不作为首选，因为 `vt100` 已更接近所需对象

### 3. Clean-Surface Policy

clean surface 必须：

- 保留 plain text 的可见内容；
- 折叠 `\r` 驱动的单行重绘历史帧；
- 解释 cursor motion / clear-line / clear-screen 后的最终可见文本；
- 去除 SGR 样式控制字节；
- 去除 OSC title 等纯元数据噪声；
- 对 OSC 8 hyperlink 保留 visible label，并将 URL 以普通文本附着；
- 对 alt-screen 保留最终 screen content，而不是保留整个 repaint transcript；
- 对 box-drawing / table / tree / panel / unicode symbol 保持正文地位，不把它们误删为噪声。

### 4. Search Parity

`search_text(path="pueue-log:<id>")` 与 `read_file(relative_path="pueue-log:<id>")` 必须消费同一 clean surface。

这条 parity 是 candidate locked element，不属于可选实现细节。

## Candidate Algorithmic Shape

### Entry

算法输入是 task-log raw substrate。
本页不把 substrate 获取方式写成主语义；
它可能来自 runtime memory、buffer file 或 server-owned adapter，只要下游看到的是同一 raw substrate 即可。

### Pre-Transform

进入 virtual screen 之前，先做 lightweight pre-scan：

- 识别 alt-screen enter / exit，按 screen session 切段；
- 识别 OSC title 与 OSC 8 hyperlink；
- 将 OSC title 降为 non-surface metadata；
- 将 OSC 8 hyperlink 重写为普通文本 `label (url)`。

这层是 task-log domain policy，不是 generic parser ontology。

### Segment Path Selection

对每个 segment：

- 若无 ESC 且无 bare `\r` 重绘语义，则走 plain path；
- 否则走 screen path。

### Plain Path

- 仅统一换行与最少必要清洗；
- 不进入 virtual screen；
- 不额外加工正文。

### Screen Path

- 使用 `vt100` 构建 virtual screen；
- 提取最终可见字符结果；
- 不保留控制序列本身；
- 不输出中间 repaint 历史帧。

### Width Discipline

virtual screen 使用固定大列宽，而不是窄终端列宽。

当前 candidate default：

- cols: `512`
- rows: `256`
- scrollback: 足够大

该选择的目的不是模拟某个真实终端窗口，
而是避免 clean projection 自己制造新的自动换行噪声。

### Error Handling

- parser 失败时回退到最保守的 plain-text projection；
- fallback 不得引入 narrative prose；
- fallback 不得伪造“已理解”的摘要；
- parser failure 只影响 projection，不影响 task truth。

## Candidate Validation Boundary

当前 draft 已锁定以下验证面。

### Sample Fixture Family

`playground/loommux/spike/简单命令行构建/cli_demo.py` 作为当前 canonical sample family。

它覆盖的等价类至少包括：

- plain-only line logs
- `\r` spinner / progress redraw
- `\r + ESC[2K` clear-line redraw
- ANSI SGR
- cursor-up / multi-line repaint
- OSC title
- OSC 8 hyperlink
- alt-screen enter / exit
- Rich mixed output

### Boundary Coverage

至少覆盖：

- 单次 `\r` 覆写
- 多次连续 `\r` 覆写且无中间 `\n`
- 最后一行无 trailing newline
- 宽字符 / 中日韩字符 / emoji / box-drawing 混合
- 极长行
- 只有控制序列、几乎无正文
- alt-screen 进入后立即退出

### Cause-Effect Coverage

至少覆盖：

- 有 `\r` 无 ESC
- 有 ESC style 无 cursor motion
- 有 cursor motion
- 有 OSC 8
- 有 alt-screen

### Parity Coverage

必须覆盖：

- `read_file(pueue-log:<id>)` 与 `search_text(path=pueue-log:<id>)` 的 clean parity
- raw terminal command 与 MCP clean surface 的上下层一致性

## Remaining Non-Accepted Boundary

本页仍是 candidate spec，而不是 accepted knowledge。

当前尚未进入 accepted family 的边界包括：

- clean-surface contract 是否正式进入 `10_knowledge/70_interfaces/`
- crate selection 是否上升为 accepted decision
- implementation plan 是否拆出独立 process asset
- alt-screen block 的最终外观是否需要单独的 architecture host

这些问题在 promotion 完成前，不得把本页内容当作 accepted truth 下游依赖。

## Exit Route

- 若被接纳：
  - interface 进入 `10_knowledge/70_interfaces/`
  - algorithm / module boundary 进入 `10_knowledge/50_architecture/`
  - clean-only / no-raw-in-MCP posture 进入 `10_knowledge/60_decisions/`
  - implementation / validation matrix 进入 `15_process_assets/`
- 若被放弃：迁入 `30_records/30_disposition/`
- 若被拆回前置议题：回流到 `20_deliberation/20_proposals/` 或 `10_issues/`
