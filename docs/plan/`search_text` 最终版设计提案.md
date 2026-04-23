# `search_text` 最终版设计提案

这次我不再按“保守收口版”讲，我直接按你的目标来设计：

**把 `search_text` 设计成一个对大模型友好、对作者低摩擦、能吃下全部 ripgrep 参数、并且尽可能自动猜意图的 `rg` 超集工具。**

我下面这套设计，不是“未来也许可以”，而是我现在就认为最好的方向。它的核心思想非常简单：

1. `rg_args` 全放开，不再人为封死。
2. 工具不再把用户逼回终端原始 `rg`。
3. 工具内部负责理解这些参数到底在表达什么意图。
4. 当参数能被 DocuTouch 的高信噪格式吸收时，就吸收。
5. 当参数已经明显在要求另一种结果对象时，就自动切换到更合适的输出模式。
6. 如果多组参数冲突，就退回最忠实的普通模式，但仍然留在这个工具里完成，不把人赶回终端。
7. 对作者来说，这个工具要表现得像：
   - 我就是 `rg`，但我更聪明；
   - 我会猜你的意图；
   - 我优先给你最适合大模型阅读的结果；
   - 实在不适合包装时，我就在当前工具里原样给你。

这才符合你要的“超集 + 对大模型友好 + 降摩擦到绝对低”的方向。

---

# 一、产品定义：`search_text` 不再是“受限 wrapper”，而是“带智能输出协商的 ripgrep 超集”

我建议把它的产品定义直接改成下面这句：

> `search_text` 是一个 ripgrep-compatible、LLM-friendly 的智能搜索工具。你可以把全部 ripgrep 参数放进 `rg_args`。工具会尽量保留 DocuTouch 的高信噪输出；如果参数表达了另一种更合适的结果对象，工具会自动切换到对应模式，例如 grouped、context、count、files、raw_text、raw_json，而不是要求你退回终端手动调用 `rg`。

这句话很重要，因为它把工具的身份从“我封装一下 `rg`，但你别越界”变成了：

- **我就是你的统一搜索入口；**
- **你不用在 DocuTouch 和终端 `rg` 之间来回切；**
- **你可以把真实意图直接扔给我；**
- **我来决定怎么把它变成最适合你当前任务的输出。**

你要的就是这个。

---

# 二、设计目标：不是“允许所有参数”这么简单，而是“允许所有参数且仍然高信噪”

这套设计要同时满足五件事：

## 1. 全量兼容 `rg_args`

`rg_args` 不再因为“render-shaping”而被一刀切拒绝。

## 2. 低摩擦

像 `{ref:` 这种作者天然会输入的东西，不能再因为默认 regex 语义把人绊倒。

## 3. 高信噪

只要还有机会保持 grouped discovery / grouped full / grouped context 这种高质量输出，就不要轻易退化成生硬的 raw 流。

## 4. 单入口

用户不需要再被教育成：

- 这个情况用 `search_text`
- 那个情况退回终端 `rg`

而是统一说：**搜东西就先用 `search_text`。**

## 5. 冲突不报“哲学错误”，只做“智能降级”

只要底层 `rg` 本身还能执行，这个工具就尽量给出结果，不要动不动说“这个参数不允许”。

我认为这五条就是你要的最终方向。

---

# 三、我建议的新接口：保持简单，但把“解释权”显式做出来

我建议最终接口长这样：

```text
search_text(
  query: string,
  path: string | string[],
  rg_args?: string,
  query_mode?: "auto" | "literal" | "regex",
  output_mode?: "auto" | "grouped" | "grouped_context" | "counts" | "files" | "raw_text" | "raw_json",
  view?: "preview" | "full"
)
```

## 字段解释

### `query`

搜索内容。

### `path`

搜索范围，继续保留单路径或路径数组，保持现在的 workspace / absolute / `pueue-log:<id>` 能力。

### `rg_args`

完全开放。可以放任意 ripgrep 参数。

### `query_mode`

这个字段是整个“零摩擦”设计的关键。

- `auto`：默认值。工具自动猜用户到底是在搜字面量还是 regex。
- `literal`：明确按字面量搜索。
- `regex`：明确按 regex 搜索，出错就直接按 regex 错误报。

### `output_mode`

这个字段是整个“参数放开后不失控”的关键。

- `auto`：默认值。根据 `rg_args` 和整体意图自动选择最合适结果形态。
- `grouped`：DocuTouch 经典 grouped-by-file 模式。
- `grouped_context`：按文件分组，但显示 match + context block。
- `counts`：返回计数导向结果。
- `files`：返回文件导向结果。
- `raw_text`：直接返回原生 rg 文本输出。
- `raw_json`：直接返回原生 rg JSON 事件流，不包任何外壳。

### `view`

只在 grouped 系列模式中有意义：

- `preview`
- `full`

保持兼容当前模型的阅读习惯。

---

# 四、`query_mode` 的最终语义：我完全同意你喜欢 `auto`，而且我认为它必须当默认值

我现在把它讲清楚。

## 为什么默认必须是 `auto`

因为作者和模型在搜索时，输入的往往不是“精确的 regex 意图”，而是“我脑子里那串文本”。

比如：

- `{ref:`
- `warning[`
- `foo(bar`
- `a+b`
- `path.to.value`

这些在作者脑子里很多时候都是“字面量文本”，而不是 regex。

如果默认坚持 regex-only，你就一定会继续遇到：

- 明明只是搜文本，结果因为 metacharacter 报错；
- 用户再去想“是不是要加 `-F`”；
- 模型再去切回终端 `rg`；
- 来回折腾调用次数和 token；
- 这和你要的目标完全相反。

所以我认为默认必须是：

**`query_mode = auto`**

## `auto` 的具体规则

我建议规则写死成下面这样：

### 规则 1
先按照 regex 路径尝试执行。

### 规则 2
如果 regex 编译成功，就按 regex 搜索，不做多余猜测。

### 规则 3
如果 regex 编译失败，并且用户没有显式声明 `query_mode = regex`，则自动退回 literal 搜索。

### 规则 4
如果发生 literal fallback，工具要在 grouped / counts / files 这类自定义文本模式里明确写一条 note：

```text
query_interpretation: literal_fallback
note: query failed as regex and was retried literally
```

### 规则 5
如果当前是 `raw_json` / `raw_text`，则不要额外包 note；这些模式追求原样返回。

我为什么认为这个规则最好？

因为它能同时保住三件事：

- 真正想用 regex 的人仍然能用；
- 不懂 regex 的作者不会因为 `{ref:` 这种输入被绊住；
- 工具仍然有明确、可解释、可测试的行为，而不是玄学猜测。

## `literal` 和 `regex` 的意义

你虽然要“智能猜”，但也不能把显式控制删掉。

所以：

- 高摩擦默认走 `auto`；
- 有明确意图的人可以强制 `literal`；
- 想严格 regex 的人可以强制 `regex`。

这就是最好用也最清楚的组合。

---

# 五、`output_mode` 才是这次重设计的真正核心

你前面说得很对：如果用户加了额外参数，导致 grouped 输出已经不再是最合适对象，那就不要死撑旧格式，而是应该给出新的高信噪格式。

我完全赞成。

但我把这个想法进一步系统化：

**不是“放开参数后偶尔例外”，而是正式把输出模式变成一等公民。**

## 为什么一定要有 `output_mode`

因为一旦你允许所有 `rg_args`，用户实际想要的结果对象就不再只有一种：

- 有时候他想做 discovery
- 有时候他想看上下文
- 有时候他只想知道哪些文件命中
- 有时候他只想要计数
- 有时候他要 JSON 事件流
- 有时候他要原始 text

如果你还把这些都塞在 `rg_args` 一个口子里，不把“结果对象”明确化，那么工具很快就会变得不可解释。

所以必须有 `output_mode`。

---

# 六、我建议的智能输出协商器：这是这套设计最关键的发动机

这部分我给你直接定成规则。

## 总原则

工具拿到 `query + path + rg_args + query_mode + output_mode + view` 后，先不急着执行，而是先做一次 **意图解析**。

它要回答两个问题：

1. 用户想怎么搜？
2. 用户想要什么类型的结果对象？

然后再决定最终执行和返回方式。

## 输出模式选择优先级

我建议优先级如下：

### 第一优先级：显式 `output_mode`

如果用户明确给了 `output_mode`，先尊重它。

### 第二优先级：从 `rg_args` 推断显式对象意图

如果没给 `output_mode`，就从 `rg_args` 推断。

### 第三优先级：默认使用 `grouped + view`

如果 `rg_args` 没表达特殊对象意图，就回到 grouped 模式。

---

# 七、我建议的 `rg_args` 分类法：不是禁止，而是识别它们在表达什么

这部分是整个设计能不能“既放开又清楚”的关键。

我建议把全部 `rg_args` 识别成下面五类。

## A 类：纯搜索行为参数

这类参数不改变结果对象，只改变搜索本身。

比如：

- `-F`
- `-i`
- `-s`
- `-w`
- `-x`
- `-g`
- `--glob`
- `-P`
- `--max-count`
- `--multiline`
- `--hidden`
- `--follow`
- `--type`
- `--type-not`

这些参数应该 **全部允许，并优先保留 grouped 模式**。

## B 类：grouped 可吸收的冗余输出参数

这类参数本来是原生 `rg` 的输出控制，但 grouped 模式已经天然具备等效信息，或者能自然吸收它们。

比如：

- `-n`
- `--line-number`
- `--no-heading`
- `--color never`

这些参数在 `grouped` / `grouped_context` 模式里不需要触发降级，也不需要报错，可以直接吸收。

大模型和作者都不需要知道内部把它“吃掉”了，只要结果符合意图即可。

## C 类：上下文导向参数

比如：

- `-A`
- `-B`
- `-C`
- `--after-context`
- `--before-context`
- `--context`
- 以及等价写法 `-C2`、`--context=2`

这类参数不应该再报错。

它们应该自动把 `output_mode` 推向：

**`grouped_context`**

如果没有其他更强的冲突。

## D 类：结果对象切换参数

这类参数表达的是“我要另一种结果对象”，不适合继续塞进 grouped。

比如：

- `--json` -> `raw_json`
- `-c` / `--count` / `--count-matches` -> `counts`
- `-l` / `--files-with-matches` / `--files-without-match` / `--files` -> `files`

这些参数不应该再报“不允许”；而应该被理解成“用户想要 count/files/json 这类对象”。

## E 类：原生文本优先参数

这类参数一旦出现，DocuTouch 自定义包装通常已经不再值得维护。

比如：

- `--replace`
- `--heading`
- `--type-list`
- 未来其他明显要求原生文本布局的 flag

这类参数在 `output_mode = auto` 下，我建议直接退到：

**`raw_text`**

这样最忠实，也最少惊喜。

---

# 八、每种输出模式我建议长什么样

这里我给你完整定义。

## 1. `grouped`

这是默认主路径，继续保留当前 `preview/full` 双视图。

### grouped preview

用途：first-pass discovery。

保留：

- `search_text[preview]:`
- `query:`
- `scope:`
- `files:`
- `matched_lines:`
- `matches:`
- `rendered_files:`
- `rendered_lines:`
- `omitted:`
- file blocks

如果发生 auto literal fallback，再补：

- `query_interpretation: literal_fallback`

### grouped full

用途：已经确认相关后，拿完整 grouped result。

保留现有模式。

## 2. `grouped_context`

这是我认为必须新增的模式。

它用于吸收 `-A/-B/-C` 这类参数，而不是继续把它们当“不允许”。

建议输出形态：

```text
search_text[grouped_context]:
query: alpha
scope: src
files: 2
matched_lines: 4
matches: 4
context: before=1 after=1

[1] src/a.txt (2 match lines, 4 rendered lines)
  M 12 | alpha
  C 13 | beta
  M 14 | gamma alpha
  C 15 | delta
```

这里：

- `M` 表 match
- `C` 表 context

为什么我推荐这种格式？

因为它既保住了 grouped-by-file 的高信噪，又把 context line 的身份说明白了。对大模型来说比原始 rg 上下文块更可读。

## 3. `counts`

这个模式应该承接：

- `-c`
- `--count`
- `--count-matches`
- 以及未来显式 `output_mode = counts`

建议输出形态：

```text
search_text[counts]:
query: alpha
scope: src
files: 3
matched_files: 2
matched_lines: 8
matches: 11

[1] src/a.txt | 6 matches
[2] src/b.txt | 5 matches
```

如果用户用的是 `--count`，优先展示 matched lines 计数；如果是 `--count-matches`，优先展示 match 次数。头部可以按实际类型写清楚。

为什么我不推荐直接 raw_text？

因为 count 本身就是一个很适合做高信噪 summary 的对象，完全没必要退回原始 rg 文本。

## 4. `files`

这个模式应该承接：

- `-l`
- `--files-with-matches`
- `--files-without-match`
- `--files`

建议输出形态：

```text
search_text[files]:
query: alpha
scope: src
files: 12
rendered_files: 12
mode: files_with_matches

[1] src/a.txt
[2] src/b.txt
[3] src/c.txt
```

这比 raw `rg -l` 更像一个稳定 surface，也更适合大模型继续引用。

## 5. `raw_json`

这个模式我完全同意你的要求：

**不包任何 DocuTouch 头，不包任何解释，不包任何二次格式，直接原样返回 rg JSON。**

它用于：

- 用户显式 `output_mode = raw_json`
- 或 `rg_args` 含 `--json`
- 或参数组合已经明显在要求 JSON 事件流

这是你要求里最明确的一点，我认为应当直接写进正式 contract。

## 6. `raw_text`

这个模式用于：

- 用户显式请求 `raw_text`
- 或 `rg_args` 包含需要保留原生文本布局的 flag，如 `--replace`
- 或参数组合冲突太多，继续做高层包装只会误导

这就是你的“如果冲突，就退回普通模式”，但仍然在 `search_text` 内完成。

我建议它的原则是：

- 尽量忠实于原始 rg 文本输出；
- 不做自定义头；
- 不做 grouped 包装；
- 让这个工具成为真正的单入口。

---

# 九、冲突处理：不要报参数冲突错误，而要做“最忠实降级”

你说得对，不要老是摆出一副“这个不行那个不行”的架势。

我的建议是定义一个统一原则：

> 当多组 `rg_args` 无法被当前高信噪模式忠实表达时，`search_text` 不应优先报契约错误，而应退到最忠实的可执行模式；只有底层 `rg` 本身无法执行时才报真正错误。

## 我建议的降级顺序

### 1. 能保持 grouped，就保持 grouped

### 2. 有 context 意图就升到 `grouped_context`

### 3. 有 count/files/json 这类明确对象意图，就切到对应模式

### 4. 如果混合得太复杂，例如：

- `--replace`
- `--heading`
- 多个原生格式控制同时出现

那就直接 `raw_text`

### 5. 如果含 `--json`

优先 `raw_json`

这个优先级非常好懂，也非常稳。

---

# 十、我给这套工具的正式教学语言也一起写出来

你前面说得很对：这个工具本质上也是靠 prompt / tool description 教会大模型怎么用。

所以我建议工具描述不要再用“仅允许 search-behavior flags，其他请回终端 `rg`”这种保守措辞。

我建议直接改成下面这版：

---

## 建议版工具描述

`search_text` 是一个 ripgrep-compatible、LLM-friendly 的智能搜索工具。`query` 是要搜索的文本或模式；`path` 是搜索范围，可为单个 path、path 数组或 `pueue-log:<id>`。`rg_args` 接受任意 ripgrep 参数，工具会自动推断最合适的结果对象并尽量保持高信噪输出。

默认情况下，`search_text` 优先返回 DocuTouch 风格的 grouped 结果，适合 discovery 和后续 `read_file` 阅读。当 `rg_args` 表达了更明确的输出意图时，工具会自动切换到更合适的模式，例如：

- context flags (`-A/-B/-C`) -> `grouped_context`
- count flags (`-c`, `--count`, `--count-matches`) -> `counts`
- file-list flags (`-l`, `--files-with-matches`, `--files`) -> `files`
- `--json` -> `raw_json`
- 需要保留原生 rg 布局的组合 -> `raw_text`

`query_mode` 默认为 `auto`：如果查询在 regex 模式下无法编译，工具会自动回退为 literal 搜索，以降低作者输入摩擦。你也可以显式指定 `literal` 或 `regex`。`output_mode` 默认为 `auto`，但也可以强制指定为 `grouped`、`grouped_context`、`counts`、`files`、`raw_text` 或 `raw_json`。

把它理解成“一个统一的智能 `rg` 入口”：先优先用 `search_text`，而不是在工具和终端之间来回切换。

---

我认为这个描述比现在那套“这是一个受限 wrapper，render-shaping flag 不要给我”更符合你的产品目标。

---

# 十一、我认为这套设计为什么是“最好”的，而不是“更宽松一点”

我最后把理由说透。

## 理由 1：它真正消灭了“退回终端 rg”的心智分叉

你最在意的点之一，就是别让模型在工具里搜一半，又因为某个参数被拒绝，退回终端调用普通 `rg`。

这套设计正好从根上解决了：

- 参数不再因为哲学原因被拒绝；
- 只要 `rg` 自己能跑，这个工具就尽量在内部完成；
- 所有调用都集中在 `search_text` 一处；
- token、调用次数、钱包压力都会更可控。

## 理由 2：它真正照顾了作者自然输入

`{ref:` 这种例子不是边角料，而是一个信号：

- 作者写的是文本意图；
- 工具却按 regex 严格路径惩罚他。

`query_mode = auto` 正好把这层摩擦砍掉。

## 理由 3：它不会因为“全开放参数”而变成一坨不可解释的东西

如果只是把禁止列表删了，系统很快会失控。

这套设计没有这么做。它做的是：

- 参数全开放；
- 但输出模式强约束、可解释、可测试；
- 所以开放不等于混乱。

## 理由 4：它保住了大模型阅读友好性

你不是想做一个“普通的 `rg` 代理”，你是要做一个 **比普通 `rg` 更适合模型读** 的工具。

这套设计里：

- discovery 走 grouped
- context 走 grouped_context
- count 走 counts
- file list 走 files
- machine-consumable 流走 raw_json
- 实在不适合包装时才 raw_text

也就是说，高信噪不是被放弃了，而是被扩展成一套更完整的输出 taxonomy。

## 理由 5：它和你要的产品气质一致

你想要的不是：

- 小心翼翼地说“这个不能传、那个不能传”；
- 用户再被教育成去终端执行原始命令；
- 最后工具只覆盖 happy path。

你要的是：

- 一个统一入口；
- 一个敢猜意图的工具；
- 一个真正让模型和作者舒服的搜索界面；
- 一个能吞复杂参数但不会失控的系统。

我认为上面这套就是最贴近这个气质的方案。

---

# 十二、最后一句话总结我的最终设计立场

我的最终立场非常明确：

**`search_text` 应该从“受限 grouped wrapper”升级为“单入口、全参数兼容、智能猜意图、自动协商输出对象的 ripgrep 超集工具”。**

其中最关键的三个钉子是：

1. `query_mode = auto` 默认开启，regex 失败自动 literal fallback。
2. `output_mode = auto` 默认开启，根据 `rg_args` 自动选择 grouped / grouped_context / counts / files / raw_text / raw_json。
3. 所有 `rg_args` 都允许传入；如果不能优雅包装，就在本工具内降级到最忠实模式，而不是把用户赶回终端。

如果你认可，我下一步就不再只是“讨论”，而是可以直接把这套设计往代码上落。

你如果愿意，我下一条就开始把这套最终设计拆成具体实现修改点。
