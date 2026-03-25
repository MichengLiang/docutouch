(deliberation-candidate-specs-apply-patch-line-number-assisted-locking-draft)=
# Apply Patch Line-Number-Assisted Locking Draft

## 作用域

本页保留 `apply_patch` line-number-assisted locking 的 candidate-spec 演化面。

其基础 contract 已进入 accepted interface knowledge；
本页当前继续承载 follow-on refinement，也就是 numbered evidence mode 的精化边界。

## Promotion Status

- landed contract host: {ref}`knowledge-interfaces-apply-patch-semantics`
- prompt-facing rationale host: {ref}`knowledge-decisions-apply-patch-numbered-anchor-guidance-rationale`
- rollout closeout host: `30_records/60_status/apply_patch_line_number_assist_rollout_status.md`
- active follow-on focus: default `header_only` / advanced `full` numbered-evidence mode split

## Target Accepted Family

- `10_knowledge/70_interfaces/`

## Candidate Contract Surface

当前 draft 已明确把 line-number-assisted `apply_patch` 约束为：

- 保持单一 public tool identity：`apply_patch`；
- 仍保持 patch-shaped authored envelope，而不是转向 transfer-language；
- line number assist 只服务 old-side evidence strengthening；
- public recommended form 为 single numbered anchor：`@@ N | visible text`；
- parser support 可接受更密的 numbered old-side evidence，但这不是 prompt-preferred subset；
- default numbered-evidence mode 为 `header_only`；
- `full` mode 才额外解释 body 中的 dense numbered old-side evidence；
- numbered evidence 使用 absolute 1-indexed line number 与 visible text 的 double-lock；
- `+` lines 不携带 old-side line number；
- same-file multi-chunk numbering 以当前 `Update File` action 的 original snapshot 为解释基线。

## Candidate Locked Elements

当前 draft 已经长成 spec 形态的部分包括：

- one explicit `@@` header at most per chunk 仍保持不变；
- prompt-preferred subset 使用 `@@ N | visible text` 作为唯一 canonical teaching form；
- body 中的 old-side evidence line 在 parser support set 下可选携带 `N | visible text` 前缀；
- `header_only` mode 下，body-level `N | visible text` 不进入 numbered-evidence 解释，而按 ordinary old-side text 处理；
- `full` mode 下，body-level context / removed lines 才进入 dense numbered old-side evidence 解释；
- numbered evidence 仅消费第一根 `|` 作为 line/content delimiter，后续 `|` 保留在 visible text 内；
- numbered evidence 只对 old-side matching 与 disambiguation 生效，不改变 `A/M/D` outcome model；
- added lines 在任何 mode 下都不作为 numbered old-side evidence 解释对象；
- 若 numbered evidence 与 target snapshot 不一致，failure contract 应指向 authored old-side evidence mismatch，而不是静默退回宽松猜测；
- ordinary unnumbered patch authoring 仍作为 compatibility surface 保留，除非后续另有 accepted policy 收紧。

## Three-Layer Discipline

当前 draft 继续维持三层分离：

1. parser support set
2. prompt-preferred subset
3. host escalation guidance

其中：

- parser support set 可以比 public recommendation 更宽；
- prompt-preferred subset 当前收敛为 `@@ N | visible text`；
- host escalation guidance 只在 single numbered anchor 仍不足时，才允许在 `full` mode 下转向更密的 numbered old-side evidence，而不是重新发明新对象。

## Preserved Draft Value

本页继续保留的价值包括：

- promotion 前的 parser support / prompt-preferred / host escalation 三层分离；
- line-number assist 作为 old-side evidence strengthening 而非 second-tool identity 的 draft framing；
- dense numbered old-side evidence 为什么不应进入默认 public guidance 的中间论证。
- default `header_only` / advanced `full` mode split 的 candidate-level精化。

## Exit Route

- 当前已完成主 promotion；
- 若未来 grammar 再次被重开，新的 open object 应回流到 `20_deliberation/20_proposals/` 或新的 candidate spec host；
- 若本页后续不再需要作为 pre-promotion pointer，可迁入 `30_records/30_disposition/`

## Source Basis

- {ref}`deliberation-proposals-line-locked-apply-patch-extension-direction`
- {ref}`deliberation-proposals-line-locked-apply-patch-syntax-tradeoff`
- {ref}`records-audit-apply-patch-anchor-semantics-investigation`
- {ref}`knowledge-decisions-apply-patch-numbered-anchor-guidance-rationale`
