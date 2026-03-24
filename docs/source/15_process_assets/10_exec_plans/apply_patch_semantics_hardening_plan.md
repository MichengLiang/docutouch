(process-assets-apply-patch-semantics-hardening-plan)=
# Apply Patch Semantics Hardening Plan

## Purpose

将 `apply_patch` 当前已接受的 warning-first compatibility posture 转化为可执行的 hardening sequence。

## Source Basis

- {ref}`knowledge-interfaces-apply-patch-semantics`
- {ref}`knowledge-decisions-apply-patch-warning-first-rationale`
- {ref}`knowledge-architecture-apply-patch-diagnostics-spec`

## Target Outcome

- current semantics 与 public tool docs 保持一致；
- success-path warning family 继续稳定；
- path identity、Windows semantics、same-path move、workspace correctness 获得更强 hardening；
- 后续 strict-profile debate 被推迟到 evidence 足够之后。

## Scope

- `apply_patch` compatibility notes
- overwrite-warning behavior
- path-identity hardening
- Windows and alias-path correctness
- same-path move safety

## Non-Goals

- 不重新讨论 `apply_patch` 与 `apply_splice` 的 object boundary；
- 不把当前 tolerated overwrite behavior 立即改成默认错误；
- 不扩大为新的 audit subsystem。

## Milestones And Duration

| Milestone | Target Outcome | Expected Duration | Entry Condition | Exit Condition |
| --- | --- | --- | --- | --- |
| M1 | docs and runtime semantics stay aligned | same-round | accepted semantics 已固定 | 相关 stable docs 与 tool docs 一致 |
| M2 | warning-first success path remains stable | same-round | current warning family 已存在 | warning behavior 与 summary shape 无回退 |
| M3 | path identity / Windows correctness hardening | multi-round | lower-layer patch runtime ready | alias / case / same-path regressions 受测试保护 |
| M4 | strict-policy revisit checkpoint | later | warning-first behavior 已被观察 | 是否需要 strict profile 有新 evidence |

## Dependency Strategy

- 先稳 docs/runtime parity；
- 再继续 lower-layer correctness hardening；
- 最后才进入 strict-profile revisit。

## Acceptance Strategy

- public docs 与 runtime behavior 不互相背离；
- success summary 仍保留 family-compatible `A/M/D` shape；
- warning 只在触发时出现；
- path identity / Windows / same-path move 风险继续收敛。

## Risk And Replan Triggers

- 若 warning-first UX 仍显著鼓励 overwrite misuse，应提前重评 strict policy；
- 若 path alias / same-path move 继续产生 correctness regressions，应把 hardening priority 继续上调；
- 若 docs/runtime drift 再次出现，应先停下并修合同步，而不是继续加新行为。

## Related Records

- {ref}`records-audit-apply-patch-confirmed-gaps-20260323`
- {ref}`records-status-apply-patch-contract-repair-wave-20260323`
