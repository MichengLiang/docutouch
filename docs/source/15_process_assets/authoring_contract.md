(process-assets-contract)=
# 15 Process Assets 作者契约

## 契约范围

本页裁定哪些过程性执行对象应进入 `15_process_assets/`。

## Allowed Objects

- execution plan
- work package
- agent handoff
- task matrix
- readiness plan

## Disallowed Objects

- accepted knowledge statement
- issue / proposal / candidate spec 正文
- actual record object
- generic scratch note

## Placement Rules

- 若对象主要承担 planning、handoff、matrix、readiness 或 coordination 职责，且需要稳定可构建地址，应进入当前子树。
- 若对象只回答“下一步做哪些动作”，而不承接整体执行结构，应优先进入 `20_deliberation/70_worklists/`。
- 若对象主要记录已经发生的事实、状态闭合或审查发现，应进入 `30_records/` 对应 family。
- 若对象中的局部判断开始长期裁定本地边界或规则，应将相应内容回写到 `00_meta/` 或 `10_knowledge/`。

## Dependency Discipline

- process asset 页必须显式说明其 `Source Basis`、`Related Records` 与 `Exit Route`。
- process asset 不得冒充 accepted knowledge 或替代 actual record。
- `15_process_assets/` 是 canonical host，不得把同一对象的权威版本同时散落在 worklists、records 与外部工单中。

## State Transition Rules

- execution result 进入 `30_records/`；
- action-only remainder 可派生或回写到 `20_deliberation/70_worklists/`；
- 长期裁判性内容进入 `00_meta/` 或 `10_knowledge/`；
- 不再需要维护的 process asset 应评估是否处置到 `30_records/`。
