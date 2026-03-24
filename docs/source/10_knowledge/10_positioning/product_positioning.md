(knowledge-positioning-product-positioning)=
# Product Positioning

## 作用域

本页记录 DocuTouch 当前已接纳的产品定位。

它回答：

- DocuTouch 是什么
- 它首先服务谁
- 它优先优化哪条高频工作流
- 哪些方向明确不作为当前产品目标

## 核心定位

DocuTouch 的定位不是“功能尽量多”的代码工具箱，
而是面向大模型代码代理的轻量、结构化、低摩擦基础文件工作台。

它首先提供的是 agent-native runtime surface，
而不是以人类终端交互为第一优先级的通用 shell 工具集。

## 主要消费者

DocuTouch 的主要消费者是 LLM，
而不是直接手敲命令的人类编译器或终端用户。

因此，它的接口、返回形态、错误消息与 warning 设计，
优先服务模型推理、上下文提取、补丁修复与回合节省。

## 主路径

DocuTouch 当前优先优化的高频工作流是：

1. 定位工作区
2. 查看目录
3. 读取文件
4. 以稳定文件边界提取上下文
5. 应用结构化修改
6. 接收可诊断、可回滚、可追溯的反馈

在这条主路径上，优先级高于工具数量的是：

- 语义稳定
- 交互路径短
- 输出低歧义
- 反馈可诊断

## 产品哲学

### 少而精

当前公开主路径围绕少量高频原语组织，
而不是通过扩张命令集合追求表面完整性。

### 结构化优先

DocuTouch 优先使用结构化 patch 输入、干净文件读取结果、稳定 ASCII 树输出与明确失败诊断，
而不是依赖松散的自然语言编辑接口。

### 兼容性与正确性并重

对上游 `apply_patch` 兼容面的继承，
不是机械照搬，也不是冲动式分叉。

当前 accepted posture 是：

- parser 与 match 行为尽量保持基线一致
- runtime 在文件组提交、部分成功与 diagnostics 上增强
- correctness 风险优先于风格性偏好处理

## 主形态判断

当前主产品形态是 MCP / 注入式 / agent-native 工具接口。

CLI 可以保留为调试、复现、桥接或兼容出口，
但不应反向定义整个系统的主 contract。

## 非目标

当前阶段明确不做：

- 面向人类 shell 用户优化的超大全家桶 CLI
- 功能铺得很宽的通用 IDE 替代品
- 把所有行为都做成高度 configurable 的复杂平台
- 与上游完全切断联系并重造一套 patch 语言

## 下游影响

本定位直接约束：

- `30_principles/` 中的长期产品原则
- `70_interfaces/` 中的工具 contract 设计
- `80_operations/` 中的维护、测试与同步纪律

## Source Basis

- `docs/archive2026年3月24日/product_positioning.md`
