(process-assets-apply-patch-line-number-assist-implementation-stream-matrix)=
# Apply Patch Line-Number-Assist Implementation Stream Matrix

## Scope

本页记录 `apply_patch` line-number-assisted locking rollout 的第一批 execution streams，
重点表达 stream、主要编辑面、依赖方向与当前 standing 之间的关系。

## Covered Objects

- parser / source-map stream
- runtime matching stream
- presentation / tool-doc stream
- QA / parity stream

## Matrix

| Relation Type | Source | Target | Status | Note |
| --- | --- | --- | --- | --- |
| constrained-by | parser / source-map stream | `20_deliberation/40_candidate_specs/apply_patch_line_number_assisted_locking_draft.md` | completed | canonical header form、old-side numbering legality 与 authored-blame preservation 已按 candidate spec 落地 |
| owns | parser / source-map stream | `codex-apply-patch/src/parser.rs` + `codex-apply-patch/tests/` | completed | numbered old-side evidence parsing、source-map location preservation 与 parse-time rejection 已闭合 |
| depends-on | runtime matching stream | parser / source-map stream | satisfied | runtime 已消费 raw numbered evidence string shape，并完成下游解释 |
| constrained-by | runtime matching stream | `20_deliberation/40_candidate_specs/apply_patch_line_number_assisted_locking_draft.md` + `15_process_assets/50_readiness/apply_patch_line_number_assist_acceptance_criteria.md` | completed | original-snapshot semantics、double-lock matching 与 mismatch truthfulness 已稳定 |
| owns | runtime matching stream | `codex-apply-patch/src/lib.rs` + `codex-apply-patch/tests/` | completed | numbered old-side matching、same-file snapshot discipline 与 representative failures 已闭合 |
| depends-on | presentation / tool-doc stream | runtime matching stream | satisfied | visible diagnostics、canonical examples 与 tool docs 已跟上 landed runtime contract |
| owns | presentation / tool-doc stream | `docutouch-core/src/` + `docutouch-server/tool_docs/` + `codex-apply-patch/apply_patch_tool_instructions.md` | completed | visible diagnostics、tool docs、examples 与 stable wording 已同步 |
| constrains | QA / parity stream | `10_knowledge/80_operations/testing_and_tool_admission.md` + `15_process_assets/50_readiness/apply_patch_line_number_assist_acceptance_criteria.md` | satisfied | parser/runtime/presentation/MCP/CLI 的 pass evidence 已可追溯 |
| verifies | QA / parity stream | `codex-apply-patch/tests/` + `docutouch-core/tests/` + `docutouch-server/tests/` + `example/` | completed | canonical success、negative inventory、transport parity 与 example truthfulness 已通过 |

## Boundary

本页只表达 stream-to-surface / stream-to-dependency 的 typed relation，
不重写 rollout plan 正文，也不代替单个 executor handoff。
