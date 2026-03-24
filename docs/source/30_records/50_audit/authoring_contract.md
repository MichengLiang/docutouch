(records-audit-contract)=
# 50 Audit 作者契约

## 契约范围

本页裁定哪些审查发现应进入 `audit/`。

## Allowed Objects

- audit finding
- review finding
- traceability finding
- rule violation finding

## Disallowed Objects

- issue 本体
- task list
- readiness plan
- accepted decision 正文

## Dependency Discipline

- audit 页必须指向被审对象或被审范围；
- audit finding 若引发未决问题，应将问题对象送入 `20_deliberation/`；
- audit 不应吞并后续的 change / disposition 执行动作。
