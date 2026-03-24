(knowledge-interfaces-search-text-ux-contract)=
# Search Text UX Contract

## Role

本页记录 `search_text` 的 accepted grouped-search contract。

它回答：

- `search_text` 在整体工作流中的角色；
- external contract 如何定义；
- `preview` 与 `full` 两种视图如何分工；
- grouped rendering、ranking 与 `rg_args` 边界如何保持稳定。

## Interface Position

`search_text` 是 discovery surface，不是 file reader。

它主要回答：

- 哪些文件值得下一步阅读；
- 匹配集合大致有多大；
- 哪些匹配行最能说明文件为何相关；
- 下一步应转向哪个 `read_file` 调用。

稳定 interaction loop 为：

1. `search_text` for discovery
2. `read_file` for inspection
3. `apply_patch` for modification

## External Contract

Canonical signature:

```text
search_text(
  query: string,
  path: string | string[],
  rg_args?: string,
  view?: "preview" | "full"
)
```

field semantics:

- `query`
  - required；
  - raw ripgrep search pattern；
  - 支持 ordinary / literal / regex / union search。
- `path`
  - required；
  - search scope；
  - 可以是单一路径，也可以是路径数组；
  - 数组按 union scope 处理。
- `rg_args`
  - optional；
  - 只承担 advanced search-behavior escape hatch；
  - 不得接管 render shape。
- `view`
  - optional；
  - 默认 `preview`；
  - 是唯一 intended user-facing output-mode switch。

## Scope Semantics

`path` 是 public field name；在产品语义上应理解为 `scope`。

当 `path` 为数组时：

- effective scope 是所有 file / directory roots 的并集；
- 调用者可以对已发现的子集做 second-pass search；
- 输出仍然保持单一 `scope` 词汇，而不是引入第二套 `paths` vocabulary。

## Two-View Model

### Preview

`preview` 是默认 discovery mode。

它回答：

- 我下一步该读什么？

`preview` 可以只渲染部分文件与部分匹配行，但必须显式、可核算地省略。

它应报告：

- total matched files
- total matched lines
- total matches
- rendered files
- rendered lines
- omitted files / lines when omission occurs

### Full

`full` 是 exhaustive grouped mode。

它回答：

- 这个 query 已确认相关，现在给我完整的 grouped result。

`full` 保持与 `preview` 相同的 grouped-by-file reading model，但不应故意省略 matched files 或 matched lines。

## Output Contract

Both views preserve the same high-level shape:

```text
search_text[<view>]:
query: ...
scope: ...
files: ...
matched_lines: ...
matches: ...
...

[1] file/path (...)
  <line> | <rendered snippet>
```

required header fields:

- `query`
- `scope`
- `files`
- `matched_lines`
- `matches`

conditional header fields:

- `rg_args` when non-empty
- `rendered_files` in `preview`
- `rendered_lines` in `preview`

输出应优先使用 `scope`，而不是机械回显原始输入。

## Ranking And Snippet Rendering

默认排序应优化 decision value，而不是 path-first convenience。

Recommended stable sort:

1. `matched_lines` descending
2. `matches` descending
3. `path` ascending

snippet rendering contract:

- `preview` 渲染 representative matching lines；
- 每个文件块只显示少量 line entries；
- 若同文件仍有更多匹配行，应显式提示；
- 极长行允许围绕 submatch span 只渲染 bounded window，但 clipping 必须显式；
- `full` 仍保持 grouped rendering，而不是退化成 raw `rg`。

## `rg_args` Taxonomy

`rg_args` 只接受 search-behavior flags。

可接受示例：

- `-F`
- `-i`
- `-g '*.rs'`
- `-P`
- `--max-count`

render-shaping flags 由 `search_text` 自身保留，不得透传。

应拒绝的典型集合包括：

- `--json`
- `--heading`
- `--no-heading`
- `--color`
- `-n`
- `--line-number`
- `-N`
- `--no-line-number`
- `-c`
- `--count`
- `--count-matches`
- `-l`
- `--files-with-matches`
- `--files-without-match`
- `--files`
- `--type-list`
- `--replace`
- `-A`
- `-B`
- `-C`
- `--context`
- `--before-context`
- `--after-context`

这条边界表达的是：

- `rg_args` 改变 how search is performed；
- `search_text` 自己决定 how results are rendered。

## Prompt-Facing Guidance

工具描述应直接教学以下 mental model：

- `search_text` 是 grouped ripgrep wrapper for the common path；
- `rg_args` 只用于 search-behavior flags；
- render-shaping flags 由 `search_text` 保留；
- raw terminal `rg` 仍然可作为 unrestricted escape hatch；
- `preview` 用于 overview，`full` 用于 exhaustive grouped output。

## Non-Goals

本页不试图承担：

- unrestricted raw-search workflow；
- every possible ripgrep customization；
- deep semantic ranking heuristics；
- host-level giant-codebase defensive limits。
