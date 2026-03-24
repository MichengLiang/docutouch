(process-assets-readiness-contract)=
# 50 Readiness 作者契约

## 契约范围

本页裁定哪些 readiness coordination objects 应进入 `50_readiness/`。

## Allowed Objects

- readiness plan
- gate checklist
- rollout readiness page
- release-entry criteria page
- QA gate page

## Disallowed Objects

- actual readiness audit result
- generic status summary
- total execution plan

## Dependency Discipline

- readiness 页必须显式说明目标 gate、输入条件、开放风险与退出条件。
- readiness 不得把 actual audit finding 与 status closure 直接写成自身正文事实。
- 若对象主要记录已经发生的审查发现或 gate closure，应迁入 `30_records/` 对应 family。
