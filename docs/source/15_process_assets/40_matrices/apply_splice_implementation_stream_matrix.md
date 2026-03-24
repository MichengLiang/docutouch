(process-assets-apply-splice-implementation-stream-matrix)=
# Apply Splice Implementation Stream Matrix

## Scope

本页记录 `apply_splice` implementation-entry 之后的第一批 execution streams，
重点表达 stream、主要编辑面、依赖方向与当前 standing 之间的关系。

## Covered Objects

- shared substrate stream
- parser / selection stream
- runtime / diagnostics / presentation stream
- transport / tool-doc stream
- QA / parity stream

## Matrix

| Relation Type | Source | Target | Status | Note |
| --- | --- | --- | --- | --- |
| constrained-by | shared substrate stream | `10_knowledge/50_architecture/apply_splice_architecture.md` | completed | 只抽 splice-agnostic correctness substrate，并保持 runtime 语义归属不外溢 |
| owns | shared substrate stream | `codex-apply-patch/` + `docutouch-core/src/` | completed | `mutation_support` 已承载 path identity、normalization 与 affected-path merge，splice runtime 直接复用 |
| constrained-by | parser / selection stream | `10_knowledge/70_interfaces/apply_splice_spec.md` + `20_deliberation/40_candidate_specs/apply_splice_formal_semantics_draft.md` | completed | envelope、selection、omission、double-lock 已锁定 |
| owns | parser / selection stream | `docutouch-core/src/` + `docutouch-core/tests/` | completed | grammar、selection parsing、resolution 与 authored-line diagnostics 已闭合 |
| depends-on | runtime / diagnostics / presentation stream | shared substrate stream | satisfied | shared path helpers 已支撑 alias-aware grouping 与 affected-path accounting |
| depends-on | runtime / diagnostics / presentation stream | parser / selection stream | satisfied | parser / selection resolution 已作为 runtime 输入基线 |
| owns | runtime / diagnostics / presentation stream | `docutouch-core/src/` + `docutouch-core/tests/` | completed | runtime core、same-file legality、structured diagnostics、partial-success accounting 与 presentation 已闭合 |
| depends-on | transport / tool-doc stream | runtime / diagnostics / presentation stream | satisfied | outer surface 已直接消费稳定 runtime/presentation contract |
| owns | transport / tool-doc stream | `docutouch-server/src/` + `docutouch-server/tests/` + `docutouch-server/tool_docs/` | completed | CLI/MCP、tool docs 与 representative parity tests 已闭合 |
| constrains | QA / parity stream | `10_knowledge/80_operations/testing_and_tool_admission.md` + `15_process_assets/50_readiness/` | satisfied | 三层测试纪律与 recursive QA 已体现在当前验证波次 |
| verifies | QA / parity stream | `codex-apply-patch/tests/` + `docutouch-core/tests/` + `docutouch-server/tests/` | completed | parser/selection/runtime/CLI/MCP 与 doc checks 已通过 |

## Boundary

本页只表达 stream-to-surface / stream-to-dependency 的 typed relation，
不重写 implementation plan 正文，也不代替单个 executor handoff。
