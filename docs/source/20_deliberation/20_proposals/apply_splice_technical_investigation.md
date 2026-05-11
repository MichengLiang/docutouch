(deliberation-proposals-apply-splice-technical-investigation)=
# Apply Splice Technical Investigation

## 作用域

本页记录 `apply_splice` 当前尚未完全收敛的技术路线提案。

它回答：

- `apply_splice` 应如何组织其实现边界；
- 哪些现有能力可以直接复用；
- 哪些能力必须 extract-before-reuse；
- 哪些关键设计问题仍未锁定，不应被提前写成 accepted knowledge。

## Target Accepted Family

- `10_knowledge/50_architecture/`
- `10_knowledge/70_interfaces/`

## Current Proposal

当前提案不是把 `apply_splice` 并入 vendored `codex-apply-patch` grammar/runtime，
而是：

1. 保持 `apply_patch` 与 `apply_splice` 的产品身份分立；
2. 从 patch-owned lower layer 中抽取共享 mutation substrate；
3. 在 DocuTouch-owned 代码中建立独立的 splice parser、selection resolver、runtime 与 presentation。

该方向的主理由是：

- 共享 path identity / atomic commit / rollback 等 correctness substrate；
- 避免把 splice grammar 强行塞进 patch grammar；
- 降低 vendored fork 边界继续模糊的风险。

## Reuse Judgment

### Direct Reuse Candidate

- path display / scope rendering helper
- thin-adapter server / CLI registration pattern
- runtime / presentation / parity 三层测试策略

### Extract-Before-Reuse Candidate

- mutation substrate
- generic diagnostic rendering helper
- numbered excerpt codec

### Explicit Non-Reuse Set

- patch grammar and hunk parser
- diff/context replacement logic
- patch-specific sidecar schema

## Unresolved Design Set

当前仍未收敛、因此不能提前写成 stable contract 的问题包括：

- exact grammar lock
- same-file move ordering
- target existence semantics
- newline / EOF fidelity policy
- diagnostics blame model

## Current Recommendation

当前推荐的推进方式是：

1. 先完成 grammar / omission token / same-file / target existence / newline policy 的 design lock；
2. 再做 shared mutation substrate extraction；
3. 只有在这些边界收紧之后，才进入 full parser/runtime implementation。

因此，本页不是 implementation kickoff authorization，
而是 implementation-prep proposal。

## Exit Route

- 若共享 substrate 与 owned splice stack 方向被接纳，相关结论进入 `10_knowledge/50_architecture/`
- 若 executable grammar / runtime semantics 被锁定，相关对象进入 `20_deliberation/40_candidate_specs/`
- 若该方向被放弃，迁入 `30_records/30_disposition/`

## Source Basis

- `docs/archive2026年3月24日/temporary/apply_splice_technical_investigation.md`
- {ref}`knowledge-interfaces-apply-splice-spec`
