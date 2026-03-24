(deliberation-evidence-gaps-contract)=
# 60 Evidence Gaps 作者契约

## 契约范围

本页裁定哪些缺口对象应进入 `evidence_gaps/`。

## Allowed Objects

- missing warrant
- missing backing
- missing authoritative source
- missing validation condition

## Disallowed Objects

- task list
- issue definition
- proposal definition
- assumption definition

## Dependency Discipline

- evidence gap 页必须指向其服务的 target object 或 target family
- 若页的中心开始转为“接下来做什么”，应迁入 `70_worklists/`
- 若页的中心开始转为“问题尚未被界定”，应迁回 `10_issues/`

