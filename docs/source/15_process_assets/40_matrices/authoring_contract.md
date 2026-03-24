(process-assets-matrices-contract)=
# 40 Matrices 作者契约

## 契约范围

本页裁定哪些 execution-facing relation matrices 应进入 `40_matrices/`。

## Allowed Objects

- task-to-file matrix
- ownership matrix
- dependency matrix
- agent assignment matrix

## Disallowed Objects

- prose-heavy execution plan
- generic checklist
- actual status record

## Dependency Discipline

- matrix 页必须优先表达 typed relation，而不是被长篇 prose 淹没。
- 若对象主要承担 narrative coordination，应迁回 `10_exec_plans/` 或 `20_work_packages/`。
- 若对象主要承担 coverage / status 汇总，应迁到 `30_records/` 对应 family。
