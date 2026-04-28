`structural_search` 是 DocuTouch MCP 的 AST 结构查询工具。它在当前 workspace 或显式 `path` 内运行 ast-grep pattern/rule 查询，并以 pretty text 返回匹配统计、结果组、`path:line` 证据、capture 摘要、omitted 和 next。

合法 `mode`：

- `find`：运行 `pattern` 或 `rule` 查询。
- `expand`：展开 `ref` 指向的结果组。
- `around`：查看 `ref` 指向匹配或 `query=path:line` 指向位置的局部结构上下文。
- `explain_ast`：解释 `ref` 指向匹配或 `query=path:line` 指向位置的局部 AST 形状。
- `rule_test`：在小范围验证 `pattern` 或 `rule`。

引用规则：

- 可注册结果分配 `qN`。
- 结果组使用 `[N]`。
- `ref="2"` 指最近查询的 `[2]`。
- `ref="q1.2"` 指当前 MCP connection 内 `q1` 的 `[2]`。

规则边界：

- 默认输出不是 JSON。
- 证据行必须可继续交给 `read_file` 使用。
- 大结果会显示 omitted。
- 包含 fix、rewrite、replacement、apply、autofix、transform 等编辑字段的 rule 会返回 `unsupported-rule-field`。
