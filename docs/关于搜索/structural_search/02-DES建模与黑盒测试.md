# structural_search DES 建模与黑盒测试

## 1. DES 建模对象

本 DES 只建模 `structural_search` 的外部可观察行为。内部后端可以是 ast-grep CLI、ast-grep library 或测试替身；黑盒测试只断言 MCP 输入、pretty text 输出、qN 引用、错误状态和结果注册行为。

DES 不建模代码修改、替换、修复、autofix 或 rewrite。

## 2. 状态变量

### 2.1 连接状态

```text
WorkspaceState = unset | set
ConnectionState = active
RecentQuery = none | qN
RegistryState = empty | contains q1..qN
NextQueryNumber = 1..N
```

含义：

```text
WorkspaceState
  DocuTouch workspace 是否已设置。

RecentQuery
  最近一次可注册 structural_search 查询。

RegistryState
  当前 MCP connection 内已注册结果。

NextQueryNumber
  下一次成功、no-matches 或 parse-partial 查询使用的 q 编号。
```

### 2.2 查询状态

```text
ModeState = find | expand | around | explain_ast | rule_test
ScopeState = default-workspace | path-bounded | invalid
LanguageState = explicit | inferred | ambiguous | unsupported
PatternState = absent | simple-pattern | rule-object | invalid | unsupported-field
FileSetState = unselected | selected | empty | invalid
ParseState = pending | parsed | partial | failed
MatchState = pending | matches | no-matches | too-many
GroupState = none | grouped | expanded
CaptureState = none | summarized | expanded
ResultState =
  success
  | no-matches
  | parse-partial
  | invalid-pattern
  | invalid-ref
  | ambiguous-language
  | unsupported-language
  | unsupported-rule-field
  | scope-error
  | workspace-required
  | unavailable
```

### 2.3 输出状态

```text
OutputSurface = pretty-text | raw-debug
OmissionState = none | groups-omitted | matches-omitted | parse-coverage-omitted
NextState = present | missing
EvidenceState = none | path-line | path-line-context | ast-node-tree
```

默认输出必须满足：

```text
OutputSurface = pretty-text
NextState = present
```

## 3. 事件集合

通用事件：

```text
StructuralSearchCalled
WorkspaceResolved
ScopeResolved
LanguageResolved
PatternResolved
RuleResolved
UnsupportedRuleFieldDetected
FileSetSelected
SourceParsed
MatchSetBuilt
MatchSetGrouped
CapturesSummarized
ReferenceResolved
ReferenceInvalid
ResultRegistered
PrettyResultReturned
```

派生事件：

```text
ExpandRequested
AroundRequested
AstExplanationRequested
RuleTrialRequested
AstSliceBuilt
LocalContextBuilt
```

错误事件：

```text
WorkspaceMissing
ScopeInvalid
LanguageAmbiguous
LanguageUnsupported
PatternInvalid
ParseFailed
ParsePartial
NoMatchesFound
TooManyMatchesFound
UnsupportedRuleFieldRejected
```

## 4. qN 分配规则

qN 分配是可测试状态迁移。

```text
success -> 分配 qN，RecentQuery=qN。
no-matches -> 分配 qN，RecentQuery=qN。
parse-partial -> 分配 qN，RecentQuery=qN。
invalid-pattern -> 不分配 qN，RecentQuery 不变。
unsupported-rule-field -> 不分配 qN，RecentQuery 不变。
invalid-ref -> 不分配 qN，RecentQuery 不变。
workspace-required -> 不分配 qN，RecentQuery 不变。
scope-error -> 不分配 qN，RecentQuery 不变。
ambiguous-language -> 不分配 qN，RecentQuery 不变。
unsupported-language -> 不分配 qN，RecentQuery 不变。
```

显式引用解析：

```text
ref=q1.2 -> query q1 group [2]
```

隐式引用解析：

```text
ref=2 -> RecentQuery.[2]
```

显式引用优先于隐式最近查询语义。

## 5. 基本流

### 5.1 基本流 A：find 成功

前置：

```text
WorkspaceState = set
mode = find
pattern 或 rule 合法
language 可选定
path/scope 有效
匹配集合非空
```

输入：

```text
{
  "mode": "find",
  "pattern": "evaluate_exec_policy($$$ARGS)",
  "path": "core",
  "language": "rust",
  "include_tests": true
}
```

事件序列：

```text
StructuralSearchCalled
  -> WorkspaceResolved
  -> ScopeResolved
  -> LanguageResolved
  -> PatternResolved
  -> FileSetSelected
  -> SourceParsed
  -> MatchSetBuilt
  -> MatchSetGrouped
  -> CapturesSummarized
  -> ResultRegistered
  -> PrettyResultReturned
```

状态迁移：

```text
RecentQuery: none -> q1
RegistryState: empty -> contains q1
ScopeState: default-workspace/path-bounded
LanguageState: explicit
PatternState: simple-pattern
ParseState: parsed
MatchState: matches
GroupState: grouped
CaptureState: summarized
ResultState: success
```

输出断言：

```text
第一行包含 structural_search[find] q1。
输出包含 pattern、language、scope。
输出包含 matches 统计。
至少一个结果组 [1]。
证据行包含 path:line。
输出包含 omitted。
输出包含 next。
next 中 expand/around 引用存在。
```

### 5.2 基本流 B：expand 成功

前置：

```text
RecentQuery = q1
q1.[1] 存在且可展开
```

输入：

```text
{
  "mode": "expand",
  "ref": "1"
}
```

事件序列：

```text
StructuralSearchCalled
  -> ExpandRequested
  -> ReferenceResolved(ref=1 -> q1.[1])
  -> CapturesSummarized(expanded)
  -> ResultRegistered
  -> PrettyResultReturned
```

状态迁移：

```text
RecentQuery: q1 -> q2
RegistryState: contains q1 -> contains q1,q2
GroupState: grouped -> expanded
CaptureState: summarized -> expanded
ResultState: success
```

输出断言：

```text
第一行包含 structural_search[expand] q2。
输出包含 from: q1.[1]。
输出包含展开后的 match 列表。
输出包含 captures。
输出包含 next。
```

### 5.3 基本流 C：显式历史引用

输入序列：

```text
q1 = find(pattern A)
q2 = find(pattern B)
expand(ref="q1.1")
```

预期：

```text
最后一次 expand 指向 q1.[1]，不是 q2.[1]。
输出 from: q1.[1]。
成功后分配 q3。
RecentQuery = q3。
```

黑盒断言：

```text
显式 qN.N 不受 RecentQuery 改变影响。
```

### 5.4 基本流 D：around 成功

前置：

```text
q1.[1] 对应至少一个 match。
```

输入：

```text
{
  "mode": "around",
  "ref": "q1.1",
  "context": ["enclosing", "siblings", "captures"]
}
```

输出断言：

```text
第一行包含 structural_search[around] qN。
输出包含 from: q1.[1]。
输出包含 Enclosing。
输出包含 Node。
输出包含 Siblings 或说明该语言/位置无 sibling 摘要。
输出包含 Captures 或说明 no captures。
证据包含 path:line。
```

### 5.5 基本流 E：explain_ast 成功

输入：

```text
{
  "mode": "explain_ast",
  "query": "core/src/safety.rs:22",
  "language": "rust"
}
```

事件序列：

```text
StructuralSearchCalled
  -> AstExplanationRequested
  -> ScopeResolved
  -> LanguageResolved
  -> SourceParsed
  -> AstSliceBuilt
  -> ResultRegistered
  -> PrettyResultReturned
```

输出断言：

```text
输出包含 structural_search[explain_ast] qN。
输出包含 source: path:line。
输出包含 language。
输出包含 node kind 或 nearest node。
输出包含 local tree。
输出包含 next。
```

### 5.6 基本流 F：rule_test 成功

输入：

```text
{
  "mode": "rule_test",
  "pattern": "SandboxPolicy::WorkspaceWrite { $$$FIELDS }",
  "path": "core/src/safety.rs",
  "language": "rust",
  "limit": 3
}
```

输出断言：

```text
输出包含 structural_search[rule_test] qN。
输出包含 status: matched 或 status: no-matches。
输出包含 test source。
如果 matched，输出包含 captures。
输出包含 next。
```

## 6. 异常流

### 6.1 workspace-required

前置：

```text
WorkspaceState = unset
path 缺失或不可解析
```

输出：

```text
structural_search[find]
status: workspace-required
```

断言：

```text
不分配 qN。
RecentQuery 不变。
输出包含 next。
```

### 6.2 ambiguous-language

前置：

```text
language 缺失
path 指向多语言文件集合
```

断言：

```text
输出 status: ambiguous-language。
输出 candidates。
不分配 qN。
next 建议显式 language 或收窄 path。
```

### 6.3 unsupported-language

前置：

```text
language 不受 ast-grep 支持
```

断言：

```text
输出 status: unsupported-language。
不分配 qN。
next 建议查看 supported language 或调整 path/language。
```

### 6.4 invalid-pattern

前置：

```text
pattern/rule 不能按选定 language 解析。
```

断言：

```text
输出 status: invalid-pattern。
输出 pattern 或 rule summary。
不分配 qN。
next 包含 explain_ast 或 simplify pattern。
```

### 6.5 unsupported-rule-field

前置：

```text
rule 包含 fix/rewrite/replacement/apply/autofix 等编辑字段。
```

断言：

```text
输出 status: unsupported-rule-field。
不分配 qN。
输出说明 structural_search 只接受查询字段。
next 建议移除编辑字段。
```

### 6.6 no-matches

前置：

```text
scope/language/pattern 合法。
匹配集合为空。
```

断言：

```text
输出 structural_search[find] qN。
输出 status: no-matches。
分配 qN。
RecentQuery=qN。
没有可展开结果组。
next 建议放宽 path 或使用 search_text。
```

### 6.7 parse-partial

前置：

```text
候选文件中一部分可解析，一部分解析失败。
存在可用匹配或可用空结果。
```

断言：

```text
输出 status: parse-partial。
分配 qN。
输出 Missing coverage。
如果存在匹配，输出结果组。
如果不存在匹配，仍说明解析缺口和 no displayed matches。
next 建议检查已展示结果或收窄 path。
```

### 6.8 invalid-ref

前置：

```text
ref 指向不存在 query 或不存在 group。
```

断言：

```text
输出 status: invalid-ref。
不分配 qN。
RecentQuery 不变。
next 建议使用可见 qN.N 或重新 find。
```

## 7. 等价类

### 7.1 find 输入等价类

```text
F1 pattern 是简单函数调用。
F2 pattern 包含单 metavariable。
F3 pattern 包含 $$$ ellipsis。
F4 rule 只含 kind。
F5 rule 含 pattern + inside。
F6 rule 含 pattern + has。
F7 rule 含 precedes。
F8 rule 含 follows。
F9 rule 含 all。
F10 rule 含 any。
F11 rule 含 not。
F12 rule 含 matches utility。
F13 rule 含 constraints。
F14 pattern 合法但无匹配。
F15 pattern 对 language 无效。
F16 rule 含 unsupported edit field。
F17 path 中仅 tests 匹配。
F18 path 中 tests 与 production 都匹配。
F19 path 中存在 generated/fixture 文件。
F20 大结果超过 limit。
```

### 7.2 expand/around 引用等价类

```text
R1 ref=N，RecentQuery 存在且 [N] 存在。
R2 ref=qN.M，历史 query 和 group 存在。
R3 ref=N，RecentQuery=none。
R4 ref=q99.1，query 不存在。
R5 ref=q1.99，group 不存在。
R6 ref 指向 no-matches 查询。
R7 ref 指向 explain_ast 结果中的 AST node。
```

### 7.3 language 等价类

```text
L1 显式 rust。
L2 显式 typescript。
L3 显式 python。
L4 单文件 path 可推断。
L5 单语言目录可推断。
L6 多语言目录 ambiguous。
L7 未支持扩展名 unsupported。
L8 显式 language 与文件扩展名不一致。
```

### 7.4 include_tests 等价类

```text
T1 include_tests=true，显示测试匹配。
T2 include_tests=false，排除测试匹配。
T3 只有 tests 有匹配，include_tests=false -> no-matches。
T4 production/tests 都有匹配，include_tests=false -> 只显示 production。
T5 include_tests=true -> 分组区分 production/tests。
```

## 8. 边界值

### 8.1 结果数量

```text
0 groups -> no-matches。
1 group -> 显示 [1]。
groups == limit -> 全部显示，omitted none。
groups > limit -> 显示 limit 组，omitted 说明剩余。
matches > per-group display limit -> 组内 omitted。
极大结果 -> 输出预算受控。
```

### 8.2 limit

```text
limit 缺失 -> 默认 limit。
limit=1 -> 只显示 1 组。
limit=0 -> parameter-error。
limit>max -> parameter-error 或 clamp，并在输出说明；推荐 parameter-error。
```

### 8.3 capture 长度

```text
短 capture -> 原样显示。
长 capture -> 使用 inline omission。
多行 capture -> 使用 compact summary。
capture 数量超过上限 -> 显示前 N 个并说明 omitted captures。
```

### 8.4 AST 局部

```text
path:line 正好在节点上 -> 返回该节点。
path:line 在空行/comment -> 返回最近 covering node 或说明 no syntax node。
line 超出文件范围 -> scope-error。
文件不可读 -> scope-error。
```

## 9. 黑盒测试矩阵

### 9.1 输出不变量测试

每个 mode 至少一条 snapshot：

```text
find success
expand success
around success
explain_ast success
rule_test matched
rule_test no-matches
```

断言：

```text
默认输出不是 JSON。
第一行包含 structural_search[mode]。
可注册结果包含 qN。
结果组编号连续。
证据行包含 path:line。
输出包含 next。
大结果包含 omitted。
派生查询包含 from。
```

### 9.2 qN 状态测试

测试 S1：隐式最近查询。

```text
q1 = find A
q2 = find B
expand 1
```

预期：`expand 1` 指 q2.[1]。

测试 S2：显式历史引用。

```text
q1 = find A
q2 = find B
expand q1.1
```

预期：展开 q1.[1]。

测试 S3：invalid-ref 不污染 RecentQuery。

```text
q1 = find A
expand q99.1 -> invalid-ref
expand 1
```

预期：最后 `expand 1` 仍指 q1.[1]。

测试 S4：no-matches 分配 qN 但不可展开。

```text
q1 = find no match
expand 1
```

预期：`expand 1` 返回 not-expandable 或 invalid-ref，RecentQuery 规则保持一致。

### 9.3 fixture 要求

开发者应准备可控 fixture workspace：

```text
Fixture A：简单函数调用。
  覆盖 pattern、metavariable、expand。

Fixture B：match arms。
  覆盖 kind、inside、siblings、around。

Fixture C：relational rules。
  覆盖 inside、has、precedes、follows。

Fixture D：composite rules。
  覆盖 all、any、not、matches。

Fixture E：constraints。
  覆盖 metavariable constraints。

Fixture F：tests/prod 混合。
  覆盖 include_tests。

Fixture G：多语言目录。
  覆盖 language inference 和 ambiguous-language。

Fixture H：parse partial。
  覆盖 parse-partial。

Fixture I：大结果。
  覆盖 limit、omitted、输出预算。

Fixture J：unsupported edit field。
  覆盖 query-only contract。
```

## 10. 完整场景测试

场景：从文本候选到结构候选再到源码阅读入口。

输入序列：

```text
1. search_text("SandboxPolicy::WorkspaceWrite", path="core")
2. structural_search find pattern="SandboxPolicy::WorkspaceWrite { $$$FIELDS }"
3. structural_search expand ref="1"
4. structural_search around ref="q2.1"
```

预期：

```text
find 输出 production/tests 分组。
expand 输出每个 match 的 captures。
around 输出 enclosing item、node、siblings。
每步都有 path:line。
每步都有 next。
qN 按顺序分配。
无任何编辑语义进入输出。
```
