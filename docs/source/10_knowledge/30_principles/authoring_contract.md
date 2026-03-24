(knowledge-principles-contract)=
# 30 Principles 作者契约

## 契约范围

本页裁定哪些 accepted high-level doctrine 应进入 `principles/`。

## Allowed Objects

- accepted principle
- accepted policy constraint
- reusable doctrine
- stable boundary rule for domain objects

## Disallowed Objects

- authoring rule
- local decision record
- unresolved principle proposal
- migration note

## Dependency Discipline

- `principles/` 以前置 positioning 与 problem-space 为 accepted framing 上游
- `principles/` 可约束多个下游家族，但不应反向依赖 `decisions/` 才成立
- 若某对象只适用于单一议题，应优先考虑进入 `60_decisions/`

