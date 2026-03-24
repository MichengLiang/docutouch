(knowledge-decisions-contract)=
# 60 Decisions 作者契约

## 契约范围

本页裁定哪些 accepted rationale objects 应进入 `decisions/`。

## Allowed Objects

- accepted decision
- accepted decision rationale
- consequence / alternatives summary
- accepted QOC / ADR result

## Disallowed Objects

- issue
- proposal
- unresolved argument
- long-lived principle
- architecture / interface / operations 主体正文

## Dependency Discipline

- `decisions/` 依赖主体家族提供被裁决对象与依据
- decision 页只应链接真实构成其 authority basis 的上游家族，不得把所有潜在上游一概挂上
- `decisions/` 可被主体家族 cross-reference，但不应反向统治主体家族的 authority
- 若对象不再是“围绕具体议题的裁决”，而是长期 doctrine，应考虑上移到 `30_principles/`
