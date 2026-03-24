(deliberation-proposals-contract)=
# 20 Proposals 作者契约

## 契约范围

本页裁定哪些候选答案对象应进入 `proposals/`。

## Allowed Objects

- candidate proposal
- candidate position
- candidate option
- candidate decomposition

## Disallowed Objects

- issue statement
- accepted decision
- candidate spec that has already grown into structured specification form
- worklist item

## Dependency Discipline

- proposal 页应指向其来源 issue 或其 target accepted family
- 若对象已具备稳定章节结构并承担 hand-off 压力，应迁入 `40_candidate_specs/`
- 若对象仅承担临时桥接作用，应迁入 `30_assumptions/`

