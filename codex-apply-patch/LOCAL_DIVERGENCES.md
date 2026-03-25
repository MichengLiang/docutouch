# Local Divergences

相对 upstream baseline，当前工作区里的 `codex-apply-patch` 是一个明确的 vendored fork。

## 1. File-group Commit Model

当前 runtime 会把相关文件操作组织为 connected file groups，并在组内保持原子提交。

这意味着：

- 关联操作不会在组内半提交
- 独立组之间仍允许 partial success

## 2. Structured Execution Report

当前 runtime 的执行结果显式区分：

- `FullSuccess`
- `PartialSuccess`
- `Failure`

失败对象还会携带更完整的 diagnostics metadata。

## 3. Diagnostics-aware Source Mapping

当前 parser / diagnostics 层保留了更细的 source-location-aware 信息，用于把 blame 更准确地指回 patch authored source。

## 4. Standalone Packaging And Tests

当前本地 fork 采用 standalone Cargo packaging，并带有 DocuTouch 自己的测试覆盖对象，例如：

- empty add file
- EOF without trailing newline
- CRLF preservation
- no-op update accounting

## 5. Numbered-Evidence Mode Split

当前本地 fork 额外引入了 numbered-evidence mode split：

- default `header_only`
- advanced `full`

这意味着：

- numbered `@@` header 默认可用
- body-level dense numbered old-side evidence 默认不解释
- body-level dense numbering 只有在显式开启 `full` mode 时才进入 runtime 解释
