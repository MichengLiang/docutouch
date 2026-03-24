# `apply_splice` 撰写纪律与证明过程

## 文档性质

本文档不是 `apply_splice` 工具正文，不是教程，不是终端帮助页，也不是一份临时 prompt。

它是一份面向团队审核与重写工作的**构造证明文档**。它的职责不是再次定义
`apply_splice` 的工具本体，而是证明：

1. 为什么当前 `apply_splice.md` 还没有长成一个足够严谨的 tool-facing semantic surface；
2. 为什么重写时不能只是“往旧文里补几条说明”；
3. 为什么未来的 `apply_splice.md` 必须按若干受控分面来组织；
4. 为什么某些内容必须进入 prompt，而某些内容即使在逻辑上为真，也不应主动写进
   prompt；
5. 为什么 omission 这类正式能力必须被展示成 canonical authored surface，而像
   `...[N chars omitted]` 这种 sampled/truncated 噪声不应被升级成模型要记忆的对象。

换句话说：

> 本文档的目标不是证明“`apply_splice` 很复杂”，而是证明“为了让模型稳定正确地
> 使用 `apply_splice`，文档必须在哪里复杂、又必须在哪里克制”。

---

## Source Basis

- `D:\MyFile\Code\experiment_1\micheng\docutouch\docutouch-server\tool_docs\apply_splice.md`
- `D:\MyFile\Code\experiment_1\micheng\docutouch\docutouch-server\tool_docs\apply_patch.md`
- `D:\MyFile\Code\experiment_1\micheng\docutouch\codex-apply-patch\apply_patch_tool_instructions.md`
- `D:\MyFile\Code\experiment_1\micheng\docutouch\docs\source\10_knowledge\70_interfaces\apply_splice_spec.md`

这些材料分别承担不同角色：

- `apply_splice_spec.md` 是 stable contract truth 的上位来源；
- `apply_patch.md` 是当前工具正文在 A 区层面的更成熟参照；
- `apply_patch_tool_instructions.md` 提供了更强的模型导向和 B 区式误读拦截参考；
- 当前 `apply_splice.md` 是这次要重写的直接对象。

---

## 总判断

当前 `apply_splice.md` 的主要问题，不是“信息量少了一点”，而是：

1. 它更像一个简短说明页，而不是一份受控语义对象；
2. 它把一些应当显式分层的职责压扁到了同一层；
3. 它没有把 stable spec 中已经成立的关键 authored surface，兑现成模型可召回的
   canonical examples；
4. 它没有充分前置高风险误读拦截；
5. 它还保留了不值得主动教学的噪声描述，污染模型的注意力预算。

因此，本次重写不能采用“修修补补”的策略，而必须先证明：

- 未来正文应该由哪些分面组成；
- 每个分面的职责是什么；
- 哪些内容必须写；
- 哪些内容不该写进 prompt 主体。

---

# 第一部分：A 区构造证明

这里的 A 区，指未来 `apply_splice.md` 中承担**本体定义、输入边界、运行语义、路径规则、
失败表面**等职责的主合同区。

## 1. 为什么 `apply_splice` 需要一个更强的 A 区

`apply_patch.md` 当前之所以读起来严谨，不是因为它篇幅长，而是因为它已经把对象拆成了
若干低歧义分面：

- Tool Identity
- Accepted Input Shape
- Minimal Grammar
- Authoring Invariants
- Execution Semantics
- Success Summary Semantics
- Compatibility Notes
- Path Rules
- Failure Surface
- Example

这些分面让模型能分清：

- 什么是对象身份；
- 什么是形式边界；
- 什么是局部硬约束；
- 什么是 runtime 一般语义；
- 什么是报告层解释；
- 什么是 intended / current runtime 之间的偏差；
- 什么是环境规则；
- 什么是失败对象学。

当前 `apply_splice.md` 的问题恰恰在于：

- 它有 identity，但太薄；
- 它有 supported surface，但没有 minimal grammar；
- 它有 selection contract，但没有 invariants；
- 它有 diagnostic family，但没有 failure surface；
- 它有 path rules，但没有把 contract 环境边界写透；
- 它有 action basis，但没有把 canonical authored shapes 真正展开给模型看。

所以当前 `apply_splice.md` 不是“错了”，而是**没有强到足以稳定驯服模型的旧经验**。

## 2. 为什么 A 区不能只保留现在这种“功能说明式写法”

如果正文只保留：

- 工具是什么；
- 支持哪些 action；
- 有哪些错误码；

那么模型会自然地把它读成一份轻量说明页，而不是强 authoring contract。

这会产生几个高频误解：

1. 把 `Append / Insert Before / Insert After / Replace` 读成并列动作，而不是 target clause；
2. 把 selection 读成“抄一段上下文”而不是严格 authored surface；
3. 把 omission 读成 sampled-view 的省略，而不是 contiguous compression token；
4. 把 `A/M/D` 读成动作回放，而不是 coarse affected-path summary；
5. 把 diagnostic code family 读成“错误名列表”，而不是 failure surface 的入口。

因此，A 区必须变成真正的 contract 区，而不能维持为“说明页”。

## 3. 未来 A 区必须至少包含哪些分面

我认为未来的 `apply_splice.md` 至少必须有下列分面，而且它们不可互相偷换职责：

### 3.1 Tool Identity

职责：

- 定义 `apply_splice` 是什么；
- 定义它不是什么；
- 定义它与 `apply_patch` 的 object boundary。

不可缺原因：

- `apply_splice` 比 `apply_patch` 更容易被误读成“另一种 patch 语法”；
- 如果不先锁 object boundary，后面所有 grammar 与 rules 都会被模型读成 patch 变体。

### 3.2 Accepted Input Shape

职责：

- 锁 outer envelope；
- 锁 action unit 的基本形状；
- 锁 delete-only 与 transfer-to-target 两类形状。

不可缺原因：

- grammar 是形式展开，但模型在 grammar 之前需要一个对象轮廓；
- 没有这一层，模型只能从 example 反推结构。

### 3.3 Minimal Grammar

这是当前 `apply_splice.md` 最明显缺失的层。

必须回答：

- source clause 长什么样；
- target clause 长什么样；
- selection block 长什么样；
- omission token 在形式上属于什么；
- whole splice program 的最小形式边界是什么。

没有这一节，模型对 authored surface 的理解仍然停留在“例子印象”，而不是形式边界。

### 3.4 Authoring Invariants

职责：

- 集中陈列不可违反的局部硬约束。

例如未来正文应当显式承载：

- transfer action 必须由 source clause + target clause 组成；
- delete action 不能再跟 target clause；
- selection 必须以 `@@` 开始；
- selection line 必须使用 exact `N | content` delimiter；
- omission token 不能错 side；
- omission token 不能连续出现；
- omission token 不能贴 selection 边界；
- selection 不能退化成 sampled/truncated inspection fragment。

如果没有这一节，这些局部硬约束会散落在 prose 里，模型召回时会非常不稳定。

### 3.5 Action Basis

这一节现在已经有雏形，但还不够强。

它不能只是“支持哪些功能”，而应明确：

- 当前完整 action basis 是什么；
- source family 与 target family 是如何组合的；
- 当前 removal family 只包含什么；
- 这是 locked basis，不是一个开放式菜单。

### 3.6 Canonical Authored Shapes

这是 omission 问题的核心。

未来正文必须至少展示：

- append shape
- anchored insert shape
- replace shape
- delete shape
- source omission example
- target omission example

理由很简单：

> 只要 omission 是 stable spec 中的正式 authored surface，它就必须被展示成模型可模仿
> 的 canonical example。

现在的 `apply_splice.md` 只承认 omission token 的存在，但没有把它做成可被模型实际调用的
surface。这就是关键问题。

### 3.7 Selection Contract

这一节不能只说：

- line-oriented
- absolute 1-indexed
- side-specific omission

它还必须说清：

- double-lock 是什么；
- omission 表示的是 contiguous compression，而不是 sampled sparse view；
- selection 是 authored denotation，不是 inspection residue；
- selection 的 visible content 是边界 identity，而不是随便摘抄的提示语。

### 3.8 Same-File And Destination Rules

这一层在 stable spec 里已经非常关键：

- same-file original snapshot
- anchored overlap illegality
- append-create
- anchored target existence
- source byte fidelity

如果这层不单独立起来，模型很容易回退到旧的文本编辑直觉，而不是 splice contract。

### 3.9 Execution Semantics

必须明说：

- copy / move / delete 的语义
- target clause 的定位语义
- connected mutation unit
- atomicity
- partial success 的存在方式

### 3.10 Success Summary Semantics

这层必须补出来。

`apply_patch.md` 已经证明：

- 只写 “returns A/M/D summary” 不够；
- 必须显式说它是 coarse summary，不是动作回放。

`apply_splice` 这里同样需要。

### 3.11 Path Rules

这层现在已经比之前好一些了，但未来重写时仍然要做到和 `apply_patch` 一样清楚：

- relative path 如何依赖 workspace / CLI current directory；
- absolute path 是否允许；
- 无 workspace 时 absolute-only 如何成立；
- path rule 属于环境 contract，不是 authoring 风格建议。

### 3.12 Failure Surface

这是当前 `apply_splice.md` 最缺的一层之一。

只列 diagnostic family，不等于定义 failure surface。

未来必须说明：

- outer/program error
- source selection failure
- target selection failure
- state failure
- overlap failure
- write failure
- partial unit failure
- blame / repair accounting 的基本结构

否则模型根本不知道这些 code 背后的运行时对象学是什么。

### 3.13 Example

Example 必须是联合锚点，而不是随手放一个最短 happy path。

它的职责是把 grammar、shape、语义和结果联合钉住。

---

## 4. A 区中明确不该主动写进去的内容

这里是关键的硬纪律。

### 4.1 不要主动写 `...[N chars omitted]`

理由：

- 它不是合法 authored surface 的正向能力；
- 它不是模型完成任务需要掌握的 productive syntax；
- 它属于 sampled/truncated inspection residue；
- 它即使在逻辑上会被拒绝，也不值得进入 prompt 的显性教学面。

这类东西一旦写进 prompt，会产生错误信号：

- 模型会觉得“这是这个工具宇宙里值得记忆的一类写法”；
- 它会挤占 omission、replace、same-file 这些真正值得记忆的 surface。

因此：

> runtime 可以拒绝它，但 prompt 不应主动教授它。

### 4.2 不要把 sampled-view 的对比逻辑写进正文

如果某些设计文档把它作为对比背景讨论，那是设计层问题；
但 tool-facing prompt 不应把这种对比逻辑前台化。

### 4.3 不要把无效形状枚举成“知识点”

未来正文应优先展示：

- 什么可以写；
- 什么是 canonical；
- 什么是稳定边界；

而不是把 prompt 预算花在“哪些奇怪写法也会被拒绝”上。

---

# 第二部分：B 区构造证明

这里的 B 区，不是第二份 A 区，不是更啰嗦的说明层，而是面向模型理解兑现的误读拦截层。

## 5. 为什么 `apply_splice` 也需要 B 区，而不是只有 `apply_patch` 需要

`apply_splice` 的对象比 `apply_patch` 更窄，但并不代表它更不需要 B 区。

恰恰相反，`apply_splice` 更容易被以下旧经验污染：

- diff patch 经验
- 普通文本编辑经验
- sampled inspection 经验
- “看到上下文就能猜语义”的经验

而这些经验一旦带进来，模型会非常自然地做出下面这些错误动作：

- 把 source/target clause 误当平级动作列表；
- 把 numbered excerpt 当成“抄一段就行”的上下文块；
- 把 omission 当成 sampled-view 的 `...`；
- 把 `apply_splice` 想成 author text tool；
- 把 `A/M/D` 理解成动作逐条回放；
- 把 same-file legality 理解成“看起来没重叠就行”。

因此，`apply_splice` 需要 B 区，不是为了多说，而是为了在模型开始“自以为懂了”之前，
把高风险误解拦下来。

## 6. 未来 B 区应该承担哪些职责

我建议未来 B 区不需要写得像理论论文，但必须承担下面几件事：

### 6.1 Operational Priorities

告诉模型先抓什么：

- 先分清 object boundary
- 再分清 source/target shape
- 再分清 selection authoring surface
- 最后才进入具体 action 构造

### 6.2 High-Risk Misreadings

集中前置拦截最危险的误解，而不是堆 FAQ。

尤其是：

- `apply_splice` 不是 author text 工具
- omission 不是 sampled `...`
- `A/M/D` 不是 verb replay
- failure code family 不是“错误名清单”

### 6.3 Failure Re-anchoring

告诉模型失败后应该如何重新锚定：

- 先重读 selection
- 再重读 target existence
- 再看 same-file legality
- 最后才考虑 runtime state

这样可以防止模型把一次失败直接归因为“工具坏了”。

### 6.4 Boundary-Preserving Judgments

这是一个收束层，告诉模型：

- 不要从 example 推出更大合法空间
- 不要从 failure prose 推断隐藏机理
- 不要从 sampled residue 反推 authored grammar

---

## 7. 为什么 B 区不能写成 FAQ，也不能写成理论堆叠

不能写成 FAQ，因为 FAQ 是按表面问题聚合，容易无限膨胀。

也不能写成理论堆叠，因为那会把工具正文需要服务的模型对象，换成服务审核者的抽象兴趣。

所以 B 区应当：

- 短于那两份 `apply_patch` 证明文档的总量；
- 强于当前 `apply_splice.md` 的误读拦截力度；
- 结构清楚，但不把后台工法全抬到前台；
- 够用即可，不做学术炫耀。

---

## 8. 未来重写的具体写作纪律

我会按下面的纪律去重写 `apply_splice.md`：

1. 先把 A 区写强，再决定是否把 B 区的一部分直接吸进正文；
2. omission 作为正式 authored surface，必须给 canonical example；
3. `...[N chars omitted]` 这类 sampled/truncated 噪声，不进入 prompt 主体；
4. 每一节只承担一种语义职责；
5. 不把 runtime guardrail 误写成模型应学习的技巧；
6. 不把设计讨论、背景比较、作者旁白写进工具正文；
7. 跟 `apply_patch.md` 保持同类 rigor，但不机械复制。

---

## 9. 结论

当前 `apply_splice.md` 的确还不够严谨，而且这个问题不是靠补几句 bullet 就能解决的。

真正的问题是：

- A 区还不够强
- omission 等正式能力还没被兑现成 canonical surface
- B 区式误读拦截还没建立
- 无效噪声还在污染 prompt 预算

因此，未来正确的推进顺序应当是：

1. 先用本文档锁定重写纪律；
2. 以 `apply_splice_spec.md` 为 stable truth；
3. 参考 `apply_patch.md` 的前台 rigor；
4. 参考 `apply_patch_tool_instructions.md` 的模型导向感；
5. 重写出一份新的、真正受控的 `apply_splice.md`。

如果审核通过，下一步就不再继续讨论，我会直接重写 `apply_splice.md`。
