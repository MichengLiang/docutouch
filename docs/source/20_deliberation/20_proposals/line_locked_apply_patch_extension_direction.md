(deliberation-proposals-line-locked-apply-patch-extension-direction)=
# Line-Locked Apply Patch Extension Direction

## 作用域

本页记录 line-locked `apply_patch` extension 的当前候选方向。

它回答：

- stronger locking 应如何进入 `apply_patch`；
- 这类扩展与 `apply_splice` 的关系应如何划界；
- 哪些设计原则已经形成方向性约束；
- 哪些问题仍处于 open design set。

## Target Accepted Family

- `10_knowledge/70_interfaces/`
- `10_knowledge/60_decisions/`

## Current Proposal

当前候选方向不是引入第二个 public patch tool，
而是在现有 `apply_patch` 身份下引入更强的 old-side locking form。

该方向当前包含三条主张：

1. 扩展改变的是 locking contract，而不是 tool identity；
2. 扩展仍保持 patch-shaped authored surface；
3. 与 `apply_splice` 保持 shared locking philosophy，但不合并 object boundary。

## Directional Principles

- preserve patch muscle memory where possible
- stronger lock is more important than decorative syntax
- stronger evidence pattern 服务于 boundary case，而不是取代 concise default
- newline / EOF preservation 必须从第一天进入设计
- same-file multi-chunk meaning 必须保持稳定

## Open Design Set

以下问题当前仍未收敛，不应被写成 accepted object：

- final tool name
- exact stronger numbered-evidence grammar
- ambiguity error contract after anchor success
- whitespace lenience level
- standalone historical binary 的最终产品地位

## Deprioritized Directions

以下方向当前被明确压低优先级，不应静默回潮：

- stacked multiple `@@` hierarchy
- separate `CTX` first-class object
- broad syntax that reopens parser/prompt drift

## Exit Route

- 若 locking extension 的 accepted rationale 被确定，相关裁决进入 `10_knowledge/60_decisions/`
- 若 parser support / prompt subset / host escalation guidance 被锁定，相关 surface 进入 `10_knowledge/70_interfaces/`
- 若该方向被放弃或替代，迁入 `30_records/30_disposition/`

## Source Basis

- `docs/archive2026年3月24日/temporary/line_locked_patch_design_record.md`
