# Line-Number Alignment Rollout Plan

## 背景与目标

当前 DocuTouch Rust 工作区在两类输出中仍存在“行号与 gutter/分隔列不对齐”的问题：

1. `apply_patch` failure diagnostics 的 blame excerpt 区域：
   - excerpt 行使用动态 `width`（随行号位数变化）；
   - 但 excerpt 上下的空 gutter 行仍是硬编码 `"   |"`；
   - 导致 `|` 列在不同位数行号场景下错位。

2. `search_text` 的 grouped-by-file 输出：
   - 每个匹配行以 `"  {line_number} | {text}"` 形式渲染；
   - 行号位数变化会导致同一文件块内 `|` 列抖动；
   - 影响扫描与定位效率。

本专项目标：

- `apply_patch` diagnostics：把 blame excerpt 视为一个 “excerpt block”，在 block 内做到严格对齐。
- `search_text`：按“文件分组块”为单位对齐行号，使每个 file block 内 `|` 列稳定。

明确不做：

- 全局对齐（跨文件块统一行号位宽），避免无谓的 padding。
- 改变工具语义，仅改输出渲染与回归测试。

## 影响范围

### 代码

- `docutouch-core/src/patch_presentation.rs`
- `docutouch-core/src/search_text.rs`
- 可能受影响的测试：
  - `docutouch-core` 单元测试
  - `docutouch-server` 的 stdio/cli smoke 中对输出文案的断言（如存在精确字符串匹配）

### 文档

本专项以代码/测试为主，不要求改动主文档。若 docs/temporary 中有旧输出样例，可保留为历史记录。

## 设计决策

### D1: `apply_patch` diagnostics 采用 excerpt block 动态 gutter

- 不再使用固定 `"   |"`。
- 当且仅当存在 source excerpt 时：
  - 在 excerpt 之前和之后插入 “动态 gutter 行”，其 `|` 列与 excerpt 的 `width` 一致。
- 若不存在 excerpt（无 source span），不强行输出 gutter 空行。

### D2: `search_text` 采用文件块内行号对齐

- 对每个 file group：
  - 先确定本次要渲染的 entries（preview: top N; full: all）。
  - 计算 `line_number_width = max(len(line_number))`（仅在当前 group 内）。
  - 渲染时使用 `{:>line_number_width$}` 做右对齐。

## 实施步骤

### Phase 0: 准备（0.5 天）

- 定位现有渲染点与相关测试断言。
- 明确哪些测试做的是“包含子串”，哪些测试做的是“精确文本”等式。

验收：

- 提交一个最小复现说明（内部注释或测试名即可），保证后续修改有回归锚点。

### Phase 1: 修复 `apply_patch` excerpt gutter（0.5 天）

- 修改 `format_patch_failure_message`：
  - 删除两处 `lines.push("   |")`。
  - 引入 helper 生成动态 gutter 行，并在 excerpt 前后插入。

验收：

- 单测覆盖：source line 位数为 1、2、3 的场景，`|` 列不再错位。
- 不破坏现有断言：`"3 | @@“`、`"| ^"` 这类核心子串仍然可匹配。

### Phase 2: 修复 `search_text` file-block 对齐（0.5 天）

- 修改 `format_search_text_result` 中渲染 entry 的代码：
  - 计算 group 内 width。
  - 输出行号右对齐。

验收：

- 新增单测：同一 file block 内包含 `9` 与 `10`（或 `9` 与 `100`）的行号时，`|` 列对齐。
- 若 server 侧有精确字符串断言，更新为新的对齐输出或改为更稳的断言策略（例如检查 `|` 列位置）。

### Phase 3: 全量测试与回归（0.5 天）

- 运行：
  - `cargo test -p docutouch-core -p docutouch-server`
- 检查：
  - `search_text preview/full` 输出仍满足 omission accounting、ranking 等 contract；
  - diagnostics 的 blame 信息未被削弱；
  - CLI/MCP parity 测试如有，按新输出同步。

验收：

- 全绿。

## 验收标准（最终）

1. `apply_patch` diagnostics 的 blame excerpt block 在不同位数行号下 `|` 列严格对齐。
2. `search_text` 的每个 file group 内 `|` 列严格对齐（preview 与 full 都成立）。
3. 不引入新语义，不改变错误码，不改变分组、排序、计数等核心 contract。
4. Rust 测试全绿：`cargo test -p docutouch-core -p docutouch-server`。

## 输出示例（仅用于直观说明）

### search_text（块内对齐）

同一 file block 内：

- `  9 | ...`
- ` 10 | ...`

应保证 `|` 的列位置一致。
