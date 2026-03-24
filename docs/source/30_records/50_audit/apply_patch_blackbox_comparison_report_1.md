(records-audit-apply-patch-blackbox-comparison-report-1)=
# Apply Patch Black-box Comparison Report 1

## Role

本页记录一轮内置编辑工具与 MCP `apply_patch` 的黑盒对比审查发现。

它回答的是：

- 在当时的实测中，两组编辑工具各自暴露了什么能力边界；
- 哪些行为差异足以构成 audit-level 风险观察；
- 哪些观察后来应进入修复、取舍或文档收紧链路。

## Source Artifact

- `docs/temporary/测试报告1.md`

## Record Scope

本记录保留 field-observation 层面的审查发现。

它不承担：

- current stable contract 的最终裁判；
- 后续修复波次的 closeout；
- 对所有测试细节的逐例重放。

## Findings Summary

该轮黑盒对比的主要发现包括：

1. ordinary edit path 上，两组工具在多数常见场景中表现接近；
2. 当时的 MCP `apply_patch` 对 empty `Add File`、CRLF preservation、`@@` authoring ergonomics 暴露出明显风险；
3. MCP `apply_patch` 具备 multi-file atomic edit、rename、delete 等内置工具不具备的能力；
4. 内置编辑工具在 empty file 与 CRLF-preserving edit 上更稳，但在 cross-file structural operation 上表达力更弱。

## Evidence Basis

- 18 组黑盒测试用例
- ordinary edit / boundary / scenario / stress comparison
- tool output 与 file-result 对照

## Downstream Effect

本记录后续可被引用于：

- contract gap 审查
- tool-boundary decision
- operations-side tool-admission judgment

## Boundary

本页是 audit finding。

它不直接宣告今天的 current contract 仍然保持测试当时的全部行为；
若后续修复已关闭其中某些 finding，本页仍保留为历史审查记录。
