(process-assets-agent-handoff-template)=
# Agent Handoff Template

## Role

本页定义 agent handoff 的最小 section 结构。

## Fixed Sections

agent handoff 默认提供：

- `Task Objective`
- `Read These First`
- `Allowed Edit Surface`
- `Disallowed Areas`
- `Exact Deliverable`
- `Verification Criteria`
- `Escalation Conditions`
- `Report-Back Format`

## Required Discipline

handoff 至少必须同时给出：

- 阅读顺序
- 可编辑边界
- 禁改边界
- 交付物
- 验收方式
- 何时停止并上报

## Section Duties

### `Allowed Edit Surface`

显式列出当前 executor 可以改动的文件、目录或对象边界。

### `Disallowed Areas`

显式列出不应顺手扩张进去的区域。

### `Verification Criteria`

定义完成标准与验证动作，而不是仅写“完成即可”。

### `Report-Back Format`

约束 executor 的回报格式，
防止结果只以口头碎片形式返回。

### `Escalation Conditions`

定义哪些情况必须停下并回报，而不是让 executor 自行扩 scope。

## Boundary

agent handoff 不应退化为：

- 口头聊天摘要；
- total execution plan；
- action queue；
- actual status record。

## Warning Cases

以下情况应被视为 handoff 不合格：

- 只说目标，不说禁改区域
- 只说改哪里，不说如何验收
- 只说“有问题再问”，却不给具体 escalation trigger
- 交付物要求与验证标准互相脱节
