(records-audit-apply-patch-confirmed-gaps-20260323)=
# Apply Patch Confirmed Gaps 2026-03-23

## Role

本页记录 2026-03-23 时点已经确认的 `apply_patch` contract gaps。

它回答的是：

- 哪些缺口已经被确认，而不是仍停留在推测；
- 这些缺口的证据落在哪里；
- 这些缺口各自指向怎样的修复类别。

## Record Scope

本记录仅覆盖 confirmed gaps。

它不承担：

- 最终产品裁决；
- future hardening sequence；
- implementation closeout state。

## Findings Summary

当前记录的 confirmed gaps 包括：

1. empty `Add File` 在本地被错误地拒绝；
2. upstream main 已允许 zero-line `Add File`，本地 strictness 不是兼容性必需；
3. existing-file update 会改写 newline style；
4. existing-file update 会改写 EOF-without-newline state；
5. active docs 教授 multi-`@@`，但 parser 不支持；
6. injected tool doc 持续放大这一 prompt/runtime drift。

## Evidence Basis

- local parser / runtime / presentation / server tests
- vendored upstream-style tests
- 2026-03-23 对 public `openai/codex` main 的对照检查
- local black-box repro

本记录的价值在于“确认已成立的缺口”，而不是讨论所有可能原因。

## Downstream Effect

本记录直接下游包括：

- {ref}`records-status-apply-patch-contract-repair-wave-20260323`
- {ref}`process-assets-apply-patch-semantics-hardening-plan`

若某个 finding 后续被关闭，本页仍作为 audit record 保留，不转写为 accepted knowledge。

## Boundary

本页是 actual record object。

它不重写：

- accepted warning-first rationale；
- current interface semantics；
- 具体 repair wave 的 gate 闭合状态。
