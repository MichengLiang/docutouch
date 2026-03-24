(deliberation-candidate-specs-apply-splice-formal-semantics-draft)=
# Apply Splice Formal Semantics Draft

## 作用域

本页记录 `apply_splice` 当前已长成 specification 形态、但尚未进入 accepted knowledge 的 formal semantics draft。

## Target Accepted Family

- `10_knowledge/70_interfaces/`

## Candidate Contract Surface

当前 draft 已明确把 `apply_splice` 约束为：

- structural-operation primitive over existing spans
- no inline authored replacement text
- source/target 选择基于 numbered line + visible content 的 double-lock validation
- transfer / move / delete semantics 基于 validated source interval 与 target range
- connected mutation units atomic commit
- `A/M/D`-style affected outcome accounting

## Candidate Locked Elements

当前 draft 已经长成 spec 形态的部分包括：

- canonical authored grammar envelope
- selection-line and omission-token well-formedness rules
- formal file / selection model
- target-range semantics
- byte-span / newline fidelity rules
- same-file original-snapshot rule and overlap rejection
- commit-unit construction and partial-success model
- missing-target policy

## Remaining Non-Accepted Boundary

本页仍然停留在 candidate spec，
因为它尚未完成最终接纳。

当前仍需由后续裁决确认的边界包括：

- draft grammar 是否成为最终 public contract
- 诊断 code family 与 blame model 的最终稳定形态
- 与 implementation/readiness evidence 的闭环是否足够

## Exit Route

- 若被接纳，相关 contract 进入 `10_knowledge/70_interfaces/`
- 若被拆回更高层方向选择，回流到 `20_deliberation/20_proposals/`
- 若被放弃或替代，迁入 `30_records/30_disposition/`

## Source Basis

- `docs/archive2026年3月24日/temporary/apply_splice_closure/formal_semantics_draft.md`
- {ref}`knowledge-interfaces-apply-splice-spec`
