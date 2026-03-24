(knowledge-operations-testing-and-tool-admission)=
# Testing And Tool Admission

## 作用域

本页记录 DocuTouch 当前的测试维护纪律与新增工具准入标准。

它回答：

- 边界修复后测试应怎样补
- 三层测试各自承担什么职责
- 何时应拒绝或延后新增工具

## 回归测试义务

若某次修复的价值来自边界行为，
则不能只改代码，必须补回归测试。

典型边界包括：

- patch 语义边界
- 事务边界
- Windows 路径边界
- alias path 边界
- success-with-warning 场景

## 三层测试分工

- `codex-apply-patch`：底层语义、路径、提交模型与 warning 检测
- `docutouch-core`：结构化映射与 outcome 表达
- `docutouch-server`：端到端工具消息、MCP 接口与用户可见 UX

测试不得全部堆在最外层，
也不得只做最内层验证而缺失端到端回归。

## 成功消息变更的额外要求

若新增行为会改变成功消息文本或 warning 呈现，
则必须补 `docutouch-server` 侧集成测试。

原因是模型实际看到的消息本身就是 contract 的一部分。

## 新增工具准入标准

在考虑新增工具前，应先回答：

1. 是否服务高频主路径
2. 能否由现有工具组合得到
3. 是否会引入新的提示词负担
4. 是否会让工具边界变得模糊
5. 是否更适合作为次级接口而非核心工具

若这些问题多数不能明确回答，
则不应急于引入新工具。

## Source Basis

- `docs/archive2026年3月24日/maintainer_guide.md`
