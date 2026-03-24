(records-status-apply-patch-contract-repair-wave-20260323)=
# Apply Patch Contract Repair Wave 2026-03-23

## Role

本页记录 2026-03-23 这一波 `apply_patch` contract repair 的执行状态与 closeout 结果。

## Source Basis

- {ref}`records-audit-apply-patch-confirmed-gaps-20260323`
- `docs/archive2026年3月24日/temporary/apply_patch_anchor_semantics_investigation.md`

## Repair Scope

本波次处理的 repair set 包括：

1. allow empty `Add File`
2. preserve existing-file newline style during update
3. preserve existing-file EOF newline state during update
4. remove active prompt/runtime drift around stacked multiple `@@`
5. repair standalone transport parity debt where needed
6. add durable regression protection

## Current State

| Gate Item | Current Standing | Note |
| --- | --- | --- |
| empty `Add File` contract repaired | closed | 已从 malformed rejection 调整到新本地合同 |
| newline style preservation | closed | existing-file update 不再静默改写 newline style |
| EOF newline preservation | closed | patch 不再无语义必要地补尾换行 |
| active `@@` contract drift | closed | prompt-facing guidance 已收紧到 parser-backed contract |
| server-facing regression protection | closed | repaired contract 已进入 durable tests |
| standalone transport parity | partial | raw standalone surface 仍需持续保持对齐 |

## Closed Outcomes

- current local contract 不再把 empty `Add File` 当成 malformed；
- existing-file update 不再把 newline / EOF state 当作副作用重写；
- prompt/runtime drift 已被压缩，不再继续教授 unsupported multi-`@@`；
- 当前 shared local contract 已获得回归保护。

## Residual Note

- vendored raw standalone `apply_patch` binary 仍是独立 transport surface；
- 它的 outward UX 需要持续对齐，但这不改变本波次 shared local contract 已闭合的事实。

## Boundary

本页是 status record。

它不承担：

- confirmed gaps 审查本体；
- accepted rationale 正文；
- 后续 strict-policy revisit 的未来计划。
