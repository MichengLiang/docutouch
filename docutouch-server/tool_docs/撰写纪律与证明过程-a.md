下面我直接给你两样东西，不再继续做外围讨论。

第一部分是**映射表**。它不是摘要，不是导读，而是一个“分面—职责—必要性—不可合并性—删除后损失—写作纪律”的严格说明表。你可以把它当成这份文档的“构造证明”。

第二部分是**完整版本**。我会按映射表的逻辑直接写成一份可审核文本。它的目标不是“好看”，而是满足以下约束：

* 面向大语言模型，而不是人类入门者；
* 只保留承担语义职责的单元；
* 保留对象的本质复杂度；
* 压缩偶然复杂度；
* 每个标题都是一个语义分面容器；
* 每段只承担一种话语职责；
* 受控词汇稳定，不做修辞性漂移；
* 一般规则、局部硬约束、兼容性偏差、环境规则、失败报告严格分离。

我先给一个总前提，免得你审核时误会我的立场：

> 这份文档不是教程，不是 prompt，不是产品介绍，不是命令行帮助页。它是一份受控语义制品。它的唯一职责，是向大语言模型注入一个低歧义、可召回、边界稳定、运行时一致的工具语义结构。

---

# 第一部分：映射表

下面这张映射表，是我对整份文档“为什么要这样构造”的逐层证明。
我不把它写成松散评论，而写成一个严格的责任矩阵。

---

## 0. 顶层构造原则

在进入具体模块前，先给出整份文档的总构造原则。这个原则不是正文的一部分，而是正文的生成约束。

| 项    | 内容                                           |
| ---- | -------------------------------------------- |
| 对象   | `apply_patch` 工具的受控语义定义                      |
| 目标读者 | 大语言模型                                        |
| 文本类型 | 工具本体定义 + 输入约束 + 执行语义 + 报告语义                  |
| 排除对象 | 教程、自然语言 prompt、调用入口说明、用户安抚性 prose            |
| 核心目标 | 保持本质复杂度不变，压缩偶然复杂度                            |
| 写作纪律 | 术语稳定；模块职责单一；例外显式；推荐与硬约束分离；理想语义与 runtime 现实分离 |
| 评价标准 | 删除测试、合并测试、替换测试、位置测试                          |

这张总表之所以必须存在，是因为它告诉审核者：下面所有模块的设计，不是任意排布，而是受一套统一公理约束。
如果连这个总公理都没有，那么后续模块会被误看成“写作者偏好”，而不是“按职责拆分后的结构”。

---

## 1. Tool Identity

| 维度      | 说明                                                                                                                |
| ------- | ----------------------------------------------------------------------------------------------------------------- |
| 模块名     | Tool Identity                                                                                                     |
| 语义职责    | 定义工具本体；排除误识类别                                                                                                     |
| 必须回答的问题 | 这是什么；这不是什么                                                                                                        |
| 不可替代性   | 没有这一节，后续所有规则都会悬空；模型只看到一些 patch 约束，却不知道它们所附着的对象边界                                                                  |
| 不可合并性   | 不能并到 Input Shape，否则本体定义会被降格成输入说明；不能并到 Execution Semantics，否则对象会被过程化；不能并到 Compatibility Notes，否则本体会被 runtime 偶然性污染 |
| 删除后损失   | 工具的能力类型消失；“不是 prose / 不是 preview / 不是自然语言接口”等边界消失；模型会把工具误归类到相邻对象                                                  |
| 写作纪律    | 只写对象身份与排除项；不写调用通道；不写示例；不写建议                                                                                       |
| 术语纪律    | “tool”“patch-shaped input”“structural editing”一旦命名，不再漂移                                                           |

**结论**：这一节必须存在，因为它承担的是本体边界定义。边界不定义，任何约束都没有稳定归属。

---

## 2. Accepted Input Shape

| 维度      | 说明                                                                                    |
| ------- | ------------------------------------------------------------------------------------- |
| 模块名     | Accepted Input Shape                                                                  |
| 语义职责    | 定义可接受输入的高层外形                                                                          |
| 必须回答的问题 | 输入对象是什么形状；外层边界是什么；内部由什么构成                                                             |
| 不可替代性   | Grammar 只能给出形式展开，不能替代高层外形说明；没有这节，模型会看到 grammar 但缺少对象级轮廓                               |
| 不可合并性   | 不能并入 Tool Identity，因为本体定义与输入轮廓不是一回事；不能并入 Minimal Grammar，因为高层形状与形式语法属于不同抽象层           |
| 删除后损失   | 模型失去“一个 literal patch text string / 外层 Begin-End envelope / 内含 file operations”这一整体轮廓 |
| 写作纪律    | 只写外形；不写局部约束；不写 runtime 执行；不写失败                                                        |
| 术语纪律    | “literal patch text string”“patch envelope”“file operation”作为高层槽位术语固定使用               |

**结论**：这一节必须存在，因为它负责把形式文法之前的对象外观先锚定。

---

## 3. Minimal Grammar

| 维度      | 说明                                                                                                               |
| ------- | ---------------------------------------------------------------------------------------------------------------- |
| 模块名     | Minimal Grammar                                                                                                  |
| 语义职责    | 提供最低歧义的形式边界                                                                                                      |
| 必须回答的问题 | Patch、FileOp、Hunk 等对象的形式组成是什么                                                                                    |
| 不可替代性   | prose 无法等价替代 grammar；grammar 是最压缩、最精确的合法性边界编码                                                                    |
| 不可合并性   | 不能并入 Input Shape，因为 formal grammar 与对象轮廓不是同一层；不能并入 Authoring Invariants，因为 grammar 描述形式，invariant 描述生成时不得违反的局部规则 |
| 删除后损失   | 形式边界退化为自然语言近似；合法性判断的不确定性上升                                                                                       |
| 写作纪律    | 只给 grammar；不夹带 prose 解释                                                                                          |
| 术语纪律    | 非终结符命名稳定，避免自然语言改写                                                                                                |

**结论**：这一节必须存在，因为这是对象本质复杂度的一部分，而且是复杂度最优压缩形式之一。

---

## 4. Authoring Invariants

| 维度      | 说明                                                                                                       |
| ------- | -------------------------------------------------------------------------------------------------------- |
| 模块名     | Authoring Invariants                                                                                     |
| 语义职责    | 集中陈列不可违反的局部硬约束                                                                                           |
| 必须回答的问题 | 生成 patch 时哪些局部条件绝不能错                                                                                     |
| 不可替代性   | 这些规则若散落在各处，召回率下降；对模型而言，局部硬约束需要集中锚点                                                                       |
| 不可合并性   | 不能并入 Patch Writing Guidance，因为 Guidance 可能包含偏好；Invariant 必须是硬约束；不能并入 Grammar，因为 grammar 不解决“哪些地方最容易生成错误” |
| 删除后损失   | 最高频的形状错误失去集中拦截层；模型更易在局部语法上滑出合法空间                                                                         |
| 写作纪律    | 每句必须是可单独调用的刚性规则；不用修辞过渡；不用柔化措辞                                                                            |
| 术语纪律    | “requires”“must”“admits only”这类强约束动词保持明确，不改写成弱建议                                                         |

**结论**：这一节必须存在，因为它不是“教程”，而是生成合法 patch 所需的局部硬约束缓存层。

---

## 5. Anchor Precision Escalation

| 维度      | 说明                                                                                          |
| ------- | ------------------------------------------------------------------------------------------- |
| 模块名     | Anchor Precision Escalation                                                                 |
| 语义职责    | 规定 hunk 定位精度不足时的升级机制                                                                        |
| 必须回答的问题 | 默认 context 不足以唯一定位时怎么办                                                                      |
| 不可替代性   | 没有这节，模型只知道 hunk 有 context，但不知道何时、如何增强定位分辨率                                                  |
| 不可合并性   | 不能并入 Grammar，因为 grammar 只讲合法结构，不讲精度升级策略；不能并入 Authoring Invariants，因为这不是“永远必须”，而是条件触发的定位升级机制 |
| 删除后损失   | 默认 3 行上下文不唯一时，模型缺乏提高锚定精度的正规路径                                                               |
| 写作纪律    | 采用严格层级：默认 → 单级 `@@` → 多级 `@@`                                                               |
| 术语纪律    | “default context”“insufficient uniqueness”“chain multiple `@@` headers”术语稳定                 |

**结论**：这一节必须存在，因为它负责控制 hunk 的定位精度，这是 patch 可用性的关键环节。

---

## 6. Execution Semantics

| 维度      | 说明                                                                                                            |
| ------- | ------------------------------------------------------------------------------------------------------------- |
| 模块名     | Execution Semantics                                                                                           |
| 语义职责    | 定义 runtime 如何解释、组织、提交 patch                                                                                   |
| 必须回答的问题 | 多操作如何执行；顺序如何解释；哪些东西原子提交；是否允许部分成功                                                                              |
| 不可替代性   | 合法输入不等于可正确预期的执行行为；没有这节，模型只有静态格式观，没有执行模型                                                                       |
| 不可合并性   | 不能并入 Grammar，因为 grammar 不定义 runtime；不能并入 Success Summary，因为执行与报告是不同层；不能并入 Compatibility Notes，因为理想执行规则不等于偏差说明 |
| 删除后损失   | 模型无法稳定预期重复 Update、Delete 后 Add、Move 的生效时机、connected file group 等行为                                            |
| 写作纪律    | 只写一般性执行语义；不混入错误处理；不混入推荐风格                                                                                     |
| 术语纪律    | “patch order”“connected file group”“independent file group”“atomic commit”“partial success”固定使用               |

**结论**：这一节必须存在，因为 runtime 行为是对象本质复杂度的一部分，不能被示例或直觉替代。

---

## 7. Success Summary Semantics

| 维度      | 说明                                                                                   |
| ------- | ------------------------------------------------------------------------------------ |
| 模块名     | Success Summary Semantics                                                            |
| 语义职责    | 定义成功摘要中的 A/M/D 的解释边界                                                                 |
| 必须回答的问题 | 成功摘要反映什么；不反映什么                                                                       |
| 不可替代性   | 若无此节，模型会自然把 A/M/D 当作操作逐条回放；这是错误推理的高发点                                                |
| 不可合并性   | 不能并入 Execution Semantics，因为“做了什么”和“如何报告做了什么”是不同语义层；不能并入 Failure Surface，因为成功摘要不是失败诊断 |
| 删除后损失   | 输出端接口解释模糊；后续链路无法稳定理解结果标签                                                             |
| 写作纪律    | 必须同时定义正面边界与负面边界：它是粗粒度 affected-path summary，不是 verb-by-verb replay                   |
| 术语纪律    | “coarse affected-path summary”“not a verb-by-verb replay”必须固定出现                      |

**结论**：这一节必须存在，因为输出解释协议本身就是工具语义的一部分。

---

## 8. Compatibility Notes

| 维度      | 说明                                                                         |
| ------- | -------------------------------------------------------------------------- |
| 模块名     | Compatibility Notes                                                        |
| 语义职责    | 显式声明抽象用途与当前 runtime 行为之间的偏差                                                |
| 必须回答的问题 | 理想语义是什么；当前 runtime 的偏差是什么                                                  |
| 不可替代性   | 没有这节，模型会把 intended semantics 与 actual runtime behavior 混为一谈，产生过度承诺         |
| 不可合并性   | 不能并入 Execution Semantics，因为一般规则与偏差说明混写会污染对象本体；不能并入 Path Rules，因为兼容性偏差不限于路径 |
| 删除后损失   | Add File / Move to 等条目的 intended semantics 与 runtime behavior 差异被隐去        |
| 写作纪律    | 必须显式双层写法：intended / in the current runtime                                 |
| 术语纪律    | “intended”“current runtime”“replace existing file contents”这类词要稳定，不做模糊化改写  |

**结论**：这一节必须存在，因为它是防止 map-territory confusion 的必要分面。

---

## 9. Path Rules

| 维度      | 说明                                                                                             |
| ------- | ---------------------------------------------------------------------------------------------- |
| 模块名     | Path Rules                                                                                     |
| 语义职责    | 定义路径如何相对 workspace 被解释，以及缺省环境下如何失败                                                             |
| 必须回答的问题 | 相对路径如何解析；绝对路径是否允许；workspace 何以确定；无 workspace 时会怎样                                              |
| 不可替代性   | path 不是语法占位符而已，它是文件系统指向机制；没有这节，runtime 映射边界不完整                                                 |
| 不可合并性   | 不能并入 Grammar，因为 grammar 不定义解析环境；不能并入 Compatibility Notes，因为这是环境规则，不是偏差说明                       |
| 删除后损失   | workspace precedence、relative/absolute handling、无 workspace 的失败条件丢失                            |
| 写作纪律    | 只写环境与解析；不写 authoring 偏好，除非偏好明确标注为偏好而非合法性边界                                                     |
| 术语纪律    | “active workspace precedence”“relative paths resolve against”“absolute paths are accepted”必须精确 |

**结论**：这一节必须存在，因为 path 是语法与运行环境接缝处的关键对象。

---

## 10. Failure Surface

| 维度      | 说明                                                                                                                    |
| ------- | --------------------------------------------------------------------------------------------------------------------- |
| 模块名     | Failure Surface                                                                                                       |
| 语义职责    | 定义失败的阶段、粒度、保留结果与诊断输出行为                                                                                           |
| 必须回答的问题 | 失败发生在哪个层；失败后什么保持不变；部分成功如何报告；诊断携带什么证据                                                                                  |
| 不可替代性   | 若无此节，失败会被错误地扁平化成“成功/失败”二值，丢失结构性信息                                                                                     |
| 不可合并性   | 不能并入 Execution Semantics，因为失败报告不是执行规则；不能并入 Success Summary，因为成功路径与失败路径的输出语义不同                                         |
| 删除后损失   | outer-format error、file-group rollback、committed changes with later failures 等结构信息全部丢失                                   |
| 写作纪律    | 用失败类型学，而不是零散例外句                                                                                                       |
| 术语纪律    | “outer-format errors”“failing file group leaves that group unchanged”“partial failure reports”必须稳定                      |

**结论**：这一节必须存在，因为部分成功与分组回滚是本质复杂度的一部分。

---

## 11. Example

| 维度      | 说明                                            |
| ------- | --------------------------------------------- |
| 模块名     | Example                                       |
| 语义职责    | 提供语法与语义的联合锚点                                  |
| 必须回答的问题 | 一个最小但充分的 patch 实例长什么样                         |
| 不可替代性   | 没有实例，抽象规则缺乏具体挂点；模型虽可处理抽象规则，但实例能提高联合召回         |
| 不可合并性   | 不能放在前面替代定义；不能散入各节，因为实例不是类型学                   |
| 删除后损失   | 抽象 grammar、file operation、组合 patch 等缺少一个一体化样本 |
| 写作纪律    | 示例必须只展示 patch 本体，不夹带接入层说明                     |
| 术语纪律    | 示例不新增新术语，不制造正文未定义的新现象                         |

**结论**：这一节必须存在，但它必须服从前面分面，而不能反过来定义前面分面。

---

## 12. 为什么不单列“Patch Writing Guidance”作为独立主节

你会注意到，我没有把原 2 号里的 `Patch Writing Guidance` 原样保留成一个大节，而是把其中内容拆成了两个更严格的分面：

* `Authoring Invariants`
* `Anchor Precision Escalation`

原因如下。

| 维度   | 说明                                   |
| ---- | ------------------------------------ |
| 原问题  | “Guidance”这个词太宽，容易把硬约束、偏好、条件触发策略混在一起 |
| 语义风险 | 模型可能把建议误读为必要条件，或把必要条件误读为经验提示         |
| 重构理由 | 将“无条件硬约束”与“条件触发的精度升级机制”物理分离          |
| 删除收益 | 降低混杂；提高模块单一性；提高术语可控性                 |

所以不是“原 Guidance 没价值”，而是它的价值需要被拆解进更精确的容器。

---

## 13. 为什么不写“Shell command”“MCP”“入口方式”

这是一个必须显式说明的排除项，因为你已经点得很准确。

| 维度    | 说明                                     |
| ----- | -------------------------------------- |
| 被排除对象 | shell command、MCP、调用入口、transport layer |
| 排除原因一 | 这些属于接入层偶然性，不属于工具本体恒定性                  |
| 排除原因二 | 写入核心文档会使模型把工具收缩成特定壳子，而不是结构化编辑能力        |
| 排除原因三 | 接入层变化频率通常高于本体层，混写会降低文档寿命               |
| 结论    | 核心文本中不写；除非另有独立接入层文档，否则不进入本体定义          |

---

## 14. 全文写作纪律总表

这张表是为了让你审核时能看到：我后面的完整版本不是“凭感觉写的”，而是严格受控。

| 纪律   | 内容                              |
| ---- | ------------------------------- |
| 术语纪律 | 同一概念只用一个主术语，不做修辞性同义改写           |
| 模块纪律 | 一个标题只承载一个分面                     |
| 句子纪律 | 一句只承担一种主要话语职责                   |
| 边界纪律 | 一般规则、推荐、兼容性偏差、环境规则、失败报告不得混写     |
| 映射纪律 | 理想语义与当前 runtime 行为分开表述          |
| 冗余纪律 | 不写人类读者导入句，不写安抚性 prose，不写接入层偶然细节 |
| 示例纪律 | 示例只做联合锚点，不承担定义职责                |
| 位置纪律 | 高层分面在前；实例在后；偏差与失败不得抢占本体定义位置     |

映射表到这里结束。
下面进入完整版本。

---

# 第二部分：完整版本

下面这份文本，是按上面映射表直接构造的。
它的目标不是“像你们已有文档”，也不是“像官方文档”，而是尽量成为一份在结构上自洽、在术语上受控、在边界上清楚、在 runtime 事实上诚实的最终文本。

---

## `apply_patch`

Primary structural editing tool for applying concrete filesystem changes through patch-shaped input.

### Tool Identity

* Patch-shaped input only.
* The tool applies a concrete edit program to the filesystem.
* The tool is grounded in the OpenAI Codex `apply-patch` grammar and parser lineage, with a stronger file-group commit model in this runtime.
* The tool is not prose instruction input.
* The tool is not a preview mode.
* The tool is not a natural-language editing interface.

### Accepted Input Shape

* The accepted input is one literal patch text string.
* The outer patch envelope starts with `*** Begin Patch`.
* The outer patch envelope ends with `*** End Patch`.
* The patch body contains one or more file operations.
* Each file operation applies over a filesystem path and contributes to one patch execution.

### Minimal Grammar

```text
Patch := Begin { FileOp } End
Begin := "*** Begin Patch" NEWLINE
End := "*** End Patch" NEWLINE

FileOp := AddFile | DeleteFile | UpdateFile

AddFile := "*** Add File: " path NEWLINE { "+" line NEWLINE }
DeleteFile := "*** Delete File: " path NEWLINE
UpdateFile := "*** Update File: " path NEWLINE [ MoveTo ] { Hunk }

MoveTo := "*** Move to: " newPath NEWLINE

Hunk := "@@" [ header ] NEWLINE { HunkLine } [ "*** End of File" NEWLINE ]
HunkLine := (" " | "-" | "+") text NEWLINE
```

### Authoring Invariants

* Every file operation requires an explicit action header.
* `*** Add File: <path>` creates an add-shaped file operation. Every content line in its body must be a `+` line.
* `*** Delete File: <path>` creates a delete-shaped file operation. Nothing follows inside that operation.
* `*** Update File: <path>` creates an update-shaped file operation. It may be followed by `*** Move to: <new path>` and one or more hunks.
* Every hunk begins with `@@`.
* Every hunk body line must begin with exactly one of: space, `-`, or `+`.
* The patch body is patch syntax. Prose instructions are invalid input.

### Anchor Precision Escalation

* By default, prefer 3 lines of context above and 3 lines of context below each change.
* If default context does not uniquely identify the target location, add an `@@` header such as `@@ class Example` or `@@ def handler():`.
* If one `@@` header is still insufficient, chain multiple `@@` headers from outer scope to inner scope until the target location is uniquely anchored.
* When adjacent changes fall within the same local region, do not duplicate overlapping context unless additional context is required for unique anchoring.

### Execution Semantics

* A single patch may contain multiple file operations.
* Repeated `*** Update File:` blocks for the same path are applied in patch order.
* `*** Delete File:` followed later by `*** Add File:` for the same path is allowed.
* `*** Move to:` applies after the updated file content is computed.
* Connected file operations commit atomically as one file group.
* Independent file groups may still succeed when another file group fails.
* Partial success is therefore possible across independent file groups, but never inside one connected file group.
* A net-zero patch may succeed with no affected files. In that case the runtime elides filesystem writes instead of touching file timestamps.

### Success Summary Semantics

* The common success block uses compact `A/M/D` outcome tags.
* These tags are a coarse affected-path summary.
* These tags are not a verb-by-verb replay of patch instructions.
* `A` reports an add-shaped outcome for a path.
* `M` reports a modify-shaped, update-shaped, or move-shaped outcome for a path.
* `D` reports a delete-shaped outcome for a path.
* Finer distinctions belong in warnings and failure diagnostics rather than in the common success path.

### Compatibility Notes

* `*** Add File:` is intended for creating a new file.
* In the current runtime, if the target already exists as a file, its contents are replaced.
* `*** Move to:` is intended for renaming to a destination path.
* In the current runtime, if the destination already exists as a file, its contents are replaced.
* Prefer `*** Update File:` when editing an existing file.
* Prefer a fresh destination path when renaming.

### Path Rules

* Relative paths resolve against the active workspace.
* Active workspace precedence is: explicit `set_workspace`, else valid `DOCUTOUCH_DEFAULT_WORKSPACE` loaded at server startup.
* If neither exists, relative paths fail and the runtime reports that a workspace must be set or that absolute paths must be used.
* Absolute paths are accepted.
* Path strings are interpreted as filesystem paths, not shell expressions.

### Failure Surface

* Outer-format errors fail before execution begins.
* A failing file group leaves that file group unchanged.
* Independent file groups may already be committed when a later file group fails.
* Partial failure reports enumerate every committed and failed `A/M/D` path needed for safe repair; they do not compress committed or failed path accounting behind omission prose.
* Execution-time failures point back to one primary patch-source location when that mapping is available robustly.
* Selected context-mismatch failures may also include one compact target-side anchor when the runtime has strong corroborating evidence.
* Failure output should keep wording compact, but not by hiding repair-critical committed or failed path information.
* The inline failure message remains the primary repair surface, and failure-time patch source persistence may add one stable patch object when the original patch was not already file-backed.
* DocuTouch does not write audit-shaped failure sidecars, patch-run caches, or secondary JSON payloads as part of the public contract.
* Broader audit trails still belong to the Codex host, which already retains tool-call receipts.

### Example

```text
*** Begin Patch
*** Add File: docs/todo.txt
+first item
*** Update File: src/app.py
@@ def greet():
-print("Hi")
+print("Hello")
*** Delete File: obsolete.txt
*** End Patch
```

---

# 第三部分：对完整版本再做一次“必要性证明”

你说你会直接审核，所以我不在这里停。我再往前走一步，把我为什么这样写、为什么没有写某些东西、为什么没有做某些“看起来更顺”的处理，再补一层说明。这样你审的时候可以直接对照。

---

## 1. 为什么开头只有一句总定义，而且很短

开头只有一句：

> Primary structural editing tool for applying concrete filesystem changes through patch-shaped input.

这句话承担四个职责：

1. 给出对象类型：tool
2. 给出能力性质：structural editing
3. 给出作用对象：filesystem changes
4. 给出输入约束入口：patch-shaped input

它没有多余的导入，没有“you can use this tool...”，没有“this tool helps...”，没有 shell，没有 MCP，没有调用方式。
因为这些都会引入接入层偶然性或者人类教程语气。

它之所以必须短，是因为这是本体定义，不是说明段。
它之所以不能更短，是因为如果删掉 `structural`，就会损失区别于任意文本编辑的能力类型；删掉 `filesystem`，就会损失对象域；删掉 `patch-shaped input`，就会损失输入边界入口。

所以这里不是“故意短”，而是做了删除测试之后得到的最小充分句。

---

## 2. 为什么 Tool Identity 里用了多条 bullet，而不是一段 prose

因为这一节的每一条都是平行边界条件，不是连贯叙事。
用 prose 段落会引入隐性的主次关系和连接词噪声；而这里需要的是：

* 每条都可单独抽取；
* 每条都可单独否定误识；
* 平行边界条件互不从属。

例如：

* not prose instruction input
* not preview mode
* not natural-language editing interface

这三条不能压成一句“it is not a prose or preview or natural language thing”之类的句子。
那样会降低类别边界的可抽取性。
分条写，是因为每一条都承担一个排除类目的职责。

---

## 3. 为什么 Accepted Input Shape 必须与 Minimal Grammar 分开

这是很多人最容易省错的地方。
他们会觉得“既然已经有 grammar，就不用再写 input shape 了”。
但这在这里是错的。

原因是：

* Input Shape 给的是对象轮廓；
* Grammar 给的是形式展开。

“一个 literal patch text string”“由 Begin/End 包围”“包含一个或多个 file operations”是对象级拓扑；
而 `Patch := Begin { FileOp } End` 是形式级定义。
这两层不是冗余关系，而是抽象层差异。

删掉前者，后者会变成孤立 grammar；
删掉后者，前者会变成低精度 prose。
所以两者都必须存在。

---

## 4. 为什么把原来的 Patch Writing Guidance 拆成两节

因为“Guidance”这个词太大了。
它会把下面三种不同东西混在一起：

* 永远必须遵守的硬约束；
* 在默认情况下的写法偏好；
* 当默认方案不足时的升级策略。

这三者的逻辑地位完全不同。
如果混在同一节里，模型会对“必须”和“条件触发”之间的界线产生不稳定解释。

所以我把它拆成：

* `Authoring Invariants`
* `Anchor Precision Escalation`

这个拆分不是为了“更细”，而是为了让不同类型的约束住进不同语义容器。

---

## 5. 为什么 Authoring Invariants 里只保留硬约束句

因为这一节的目标不是“教怎么写得更好”，而是“拦截非法形状”。
所以这节里的句子必须满足两个条件：

1. 可以离开上下文单独成立；
2. 一旦召回，就能直接作为局部约束使用。

例如：

* Every file operation requires an explicit action header.
* Every hunk begins with `@@`.
* The patch body is patch syntax. Prose instructions are invalid input.

这些都是“可单独调用”的句子。
如果把这一节写成带大量解释的 prose，局部硬度就会被稀释。

---

## 6. 为什么 Anchor Precision Escalation 用“默认→不足→继续升级”的形式

因为这一节的职责不是陈列静态事实，而是给出一个最小决策谱系。
它必须能回答：

* 默认怎么做；
* 默认不够时怎么做；
* 一次升级还不够时怎么做。

所以层级顺序本身是语义的一部分。
如果把这三条打散到不同位置，模型就需要自己重建升级序列，这会增加偶然复杂度。

---

## 7. 为什么 Execution Semantics 中保留了 net-zero patch

因为它看起来像“很细的补充”，但其实不是。
它定义的是：

* 成功不等于一定有文件写入；
* 成功与“触碰时间戳”不是绑定关系；
* runtime 对无净变化 patch 有明确语义。

如果删掉这一条，模型可能默认“成功 patch 一定写文件”。
这会造成错误推理。
所以这句虽小，但有不可替代的边界定义价值。

---

## 8. 为什么 Success Summary Semantics 单独存在，而且反复强调“not a verb-by-verb replay”

因为这是一个高风险误读点。
只要这一层不显式写出来，模型就会自然把 `A/M/D` 当成“操作日志摘要”，甚至推断出不存在的细粒度对应关系。
所以这里必须用“正面定义 + 反面排除”的双句法：

* it is a coarse affected-path summary
* it is not a verb-by-verb replay

只写前一句不够，因为前一句没有排除最常见误解；
只写后一句也不够，因为后一句没有给出正确定义。
两句都必须在。

---

## 9. 为什么 Compatibility Notes 要用 “intended / current runtime” 的双层句式

因为这节的整个价值，就建立在“理想语义”和“当前实现”必须被并列又分离地表达。
如果只写 runtime 行为，会失去抽象用途；
如果只写 intended semantics，会失去现实边界。
所以这类条目必须成对出现：

* intended for ...
* in the current runtime ...

这不是文风选择，而是映射精度纪律的文本体现。

---

## 10. 为什么 Path Rules 里必须显式写 workspace precedence

因为 path 解析如果没有 precedence，就只剩下一句模糊的“relative to workspace”。
而“workspace”本身在 runtime 中是如何确定的，是一条高价值环境规则。
没有这条，relative path 的解释链就是断的。

这里之所以不能省，是因为你要求“本质复杂度不能因为表现形式被损失”。
workspace precedence 显然属于本质复杂度的一部分，而不是偶然细节。

---

## 11. 为什么 Failure Surface 现在不再保留 audit sidecar artifact、JSON payload 等二次工件

因为当前团队结论已经改变：失败对象仍以 inline message 为主，而审计与回执由 Codex 宿主负责。
继续在工具层写 audit sidecar / patch-run cache / JSON payload 只会形成重复缓存与重复审计。

因此这里真正要保留的不是二次审计工件，而是足够强的 inline blame evidence，加上失败时可回读的 patch source 对象本身。
拒绝 audit sidecar 不等于拒绝 failed patch source persistence；前提是持久化对象服务于下一轮模型修复，而不是变成第二套工具层审计系统。

---

## 12. 为什么 Example 放在最后，而且只保留 patch 本体

因为 Example 的职责是联合锚定，不是定义体系。
它必须晚于所有分面，否则模型会拿实例去反向猜类型学，导致例子承担本不该承担的定义职责。
同时它不能包含 shell、命令入口、调用包装，因为那会把接入层偶然信息重新注入本体文本。

所以“例子在最后、只写 patch 本体”不是简化，而是职责纯化。

---

# 第四部分：我主动指出这份完整版本仍然可以继续被你严格追问的地方

我不装作这已经是形而上的终稿。我主动告诉你：如果你要用最严格的标准继续拆，下面几个点仍然可以成为你重点审查的对象。我先把它们暴露出来，避免你觉得我在躲。

## 1. `Tool Identity` 中第一条与开头总定义之间是否存在边缘重复

我目前保留了两层：

* 标题下的一句总定义
* Tool Identity 内的 bullet

这样做的理由，是把“总定义”作为对象句，把后续 bullet 作为边界展开。
但如果你追问“这两层是否可进一步合并”，这是一个合理问题。
我当前判断是：不合并更稳，因为总定义承担对象句职责，Identity bullet 承担边界条列职责。
但这一点确实是全文中相对接近边界的位置。

## 2. `Accepted Input Shape` 与 `Minimal Grammar` 的抽象层间距是否最优

我现在保留了高层轮廓 + formal grammar 双层。
如果你要求极限压缩，可能会问：是否可用一句更强的过渡减少一条 bullet。
我当前选择不做，因为宁可保留清晰边界，也不压缩到让抽象层关系隐含化。

## 3. `Authoring Invariants` 里 “Delete File nothing follows” 是否能被 grammar 完全覆盖

从形式上讲，grammar 确实已覆盖这一点；
但我仍把它写进 invariant，因为它属于高频局部形状错误。
这相当于有意做了一次“形式已定义，但仍在人为错误高发点再显式拦截”。
这是一个经过权衡保留的重复。
它不是无意识冗余，而是为了提高局部约束召回率的功能性重复。

## 4. `Compatibility Notes` 中 “Prefer ...” 两句是否应另起 `Preference Heuristics`

严格来说，`Prefer Update File...` 和 `Prefer a fresh destination path...` 属于偏好，不是兼容性事实。
我把它们留在 Compatibility Notes，是因为它们直接跟前面 runtime 偏差相邻，形成一个局部补偿逻辑。
但如果你要求最纯粹的分面正交，这两句确实可以独立成 `Preference Heuristics`。
我当前没有单列，是出于“避免再新增分面”的压缩权衡。
这会是我认为全文中最值得继续斟酌的第二个位置。

---

# 第五部分：我的最终交付立场

我最后把立场压成一句话，方便你审：

> 我这次给你的不是“更像某种文风”的文档，而是一份按本体隔离、语义分面、受控术语、映射精度、失败对象结构化这几条纪律构造出来的受控语义文本。

如果你审核它，最值得看的不是“像不像官方”“像不像某种产品说明”，而是下面这些问题：

* 每个标题是不是都只承担一个分面？
* 每一节删除后，会不会造成清晰的语义损失？
* 术语是否稳定，没有修辞性漂移？
* 一般规则、硬约束、条件升级、运行时偏差、环境规则、失败类型学，是否都被物理隔离？
* 有没有把接入层偶然性偷偷混入本体定义？
* 有没有用 prose 平滑掉其实应当显式存在的边界？
* 有没有为了显得“短”而损失对象的本质复杂度？

如果这几条过了，这份文档才算站得住。

你可以直接按这个版本开审。
