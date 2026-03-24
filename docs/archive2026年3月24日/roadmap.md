# DocuTouch Roadmap

## 目的

这份路线图的目标不是承诺具体日期，而是明确阶段优先级、避免项目被即时想法牵着走。

路线图应当持续回答三个问题：

1. 现在最值得做什么？
2. 哪些事情以后可以做，但现在不该优先？
3. 哪些事情当前明确不做，避免方向漂移？

## 当前阶段判断

项目目前已经具备一套可用且有明显差异化价值的基础工具层：

- 文件查看
- 稳定的单文件上下文抽取
- 结构化 patch 修改
- 诊断增强
- 文件组级事务边界

因此，当前阶段的工作重点不应是功能暴涨，而应是**基础层打磨**。

最近已经落地的高价值打磨包括：

- `read_files` 退役与主路径收缩
- workspace 语义统一到 `set_workspace` / `DOCUTOUCH_DEFAULT_WORKSPACE` / relative-path error 这一条明确规则
- `apply_patch` execution failure 的 primary patch-source 定位
- failure diagnostics strengthening and inline repair UX hardening
- standalone CLI 与 server 的 no-op success 文案对齐
- 初始 `docutouch` CLI adapter 已落地，并与 MCP 共享 `search_text` / `apply_patch` 文本语义

## 近期优先级（高）

### 1. 继续加强 `apply_patch` correctness

包括但不限于：

- 路径同一性边界
- Windows case/path alias 边界
- workspace/path correctness
- same-path move 相关保护
- warning 与 diagnostics 的一致性

### 2. 保持文档、UX 与 runtime 行为同步

重点不是增加很多文案，而是防止：

- 文档还停留在旧立场
- 运行时已经改了
- warning 又没跟上

### 3. 保持工具主路径简洁

新增能力时，优先问它是否减少上下文往返和回合浪费，而不是问它是否“看起来更全”。

## 中期优先级（可做）

### 1. 更系统的 warning / diagnostics 体系

例如：

- warning code 进一步体系化
- success warning 与 failure diagnostics 更统一
- 更丰富但仍然克制的 help 信息

### 2. 移除工具层重复审计与 failure 报告工件

当前新结论：

- 审计型 patch failure artifacts / sidecars 不再属于主产品方向
- ChatGPT/Codex 不需要工具层再造一套失败报告或 JSON 回放，人类审计也更适合直接查看宿主的 tool-call 回执
- 但失败时的 patch source 本身属于修复对象；当补丁原文没有现成文件承载时，应允许在工作区隐藏目录下持久化 failed patch source，供模型继续读取
- 预算应继续优先投入到完整 repair accounting、稳定 patch 路径引用与更整齐的 diagnostics hierarchy，而不是工具层的审计感

说明：

- 宿主级日志是更高位、更完整的审计面
- DocuTouch 的职责应收敛为可直接修复的失败消息，加上失败时可回读的 patch source 对象

### 3. 对基础文件工具继续做低噪音优化

例如：

- 进一步优化多次普通读取的编排体验
- 保持文件边界稳定、重读粒度可控
- 继续控制输出中的无效噪音

### 4. 继续打磨 `search_text`

`search_text` 已经落地，当前重点不再是“是否提供这个工具”，而是继续优化它的 token 形态、可维护性与大输入边界：

- 保留 raw terminal `rg` 作为无限制逃生口
- 对高频搜索提供 grouped-by-file 的低噪音输出
- 减少重复路径刷屏
- 让搜索结果更自然地接向 `read_file`

后续设计与演进记录见：

- `docs/search_text_design.md`
- `docs/search_text_ux_contract.md`

## 中长期可探索项（暂不优先）

### 1. CLI 形态

当前立场：

- 已经存在初始 adapter
- 但仍不作为主产品形态优先推进

当前已落地范围：

- `docutouch list`
- `docutouch read`
- `docutouch search`
- `docutouch patch`
- CLI relative path 使用 CWD 作为隐式 anchor
- 搜索与 patch 的高价值文本语义已下沉到共享层，避免与 MCP 漂移

只有当以下条件明确时，CLI 才值得提升优先级：

- 有明确的非 MCP 使用场景
- 需要桥接外部 agent framework
- 调试/复现/自动化场景已经频繁受限于没有 CLI

### 2. 可选 strict profile

例如是否引入：

- `Add File` create-only strict mode
- `Move to` destination-must-be-absent strict mode

当前不优先，因为：

- 这会影响与上游兼容性
- 也会影响模型训练/使用分布
- 需要先观察 warning-first 方案是否已经足够好

### 3. 更多工具种类

当前不把“扩工具数量”作为主要方向。只有当现有工具无法优雅覆盖一个高频主路径问题时，才考虑新增工具。

## 当前明确不做的事情

### 1. 不把项目做成“什么都能干”的通用工具箱

这会直接破坏基础层的清晰边界。

### 2. 不为了表面一致而牺牲 runtime 诊断质量

如果某个返回格式更利于模型理解和自修复，就不应因为追求形式统一而削弱它。

### 3. 不在没有记录的情况下扩大与上游的行为分叉

偏离可以做，但必须有记录、有理由、有测试。

## 路线图更新规则

当下面这些事情发生时，应更新本文件：

- 优先级排序变化
- CLI 立场变化
- 新增一条明确的长期方向
- 某个暂缓项被提前
- 某个原本计划项被放弃

路线图不是“永远正确”的文件，但它必须让未来维护者看见：

- 我们当时认为最重要的是什么
- 我们为什么没有去做另一些看似诱人的事情
