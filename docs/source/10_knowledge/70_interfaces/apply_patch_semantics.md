(knowledge-interfaces-apply-patch-semantics)=
# Apply Patch Interface Semantics

## Role

本页记录 `apply_patch` 当前已接纳的 interface semantics。

它回答：

- public authoring intent 是什么；
- 当前 runtime 实际承诺了哪些语义；
- warning 与 success / failure surface 如何被调用者理解；
- 哪些行为属于兼容性语义而非推荐策略。

## Authoring Baseline

`apply_patch` 仍以 patch-shaped authored input 为基线。

调用者应优先按以下 intent 编写：

- `Add File` 用于创建新文件；
- `Update File` 用于编辑既有文件；
- `Move to` 用于把文件重命名到新的 destination path。
- 当 `Update File` 需要 auxiliary-location evidence 时，public recommended form 为 `@@ N | visible text`；
- line-number assist 只服务 old-side evidence strengthening，而不是改变 tool identity；
- line-number assist 是 optional disambiguation aid，而不是默认必写负担；
- raw textual `@@ def handler():` / `@@ class Example` 若仍被 parser 接受，也属于 compatibility surface，而不是当前 canonical public guidance。

authoring guidance 应保持狭窄，不把 runtime tolerance 倒过来教成 preferred tactic。

## Current Runtime Semantics

当前 runtime 已承诺的语义包括：

- `Add File` 命中既有文件时，会替换该文件内容；
- `Move to` 命中既有目标文件时，会替换目标文件内容；
- numbered old-side evidence 以 absolute 1-indexed line number 与 visible text 形成 double-lock；
- `@@ N | visible text` 不再被当作普通字符串 header 去全文件模糊搜索，而是先按 numbered anchor 解释；
- default numbered-evidence mode 为 `header_only`，即默认只解释 numbered `@@` header；
- `full` mode 下，body 中更密的 numbered old-side evidence 才按 old-side lock 解释；
- added lines 在任何 mode 下都不作为 numbered old-side evidence 解释对象，而按 ordinary new text 保留；
- numbered old-side evidence 以当前 `Update File` action 的 original snapshot 为解释基线；
- repeated `Update File` blocks on the same existing path 按出现顺序应用；
- `Add File` 后接同路径 `Update File` 可以在同一 commit unit 内工作；
- connected file groups 保持原子性；
- independent file groups 允许 partial success；
- net-zero `Update File` 可以成功且不触碰文件时间戳；
- partial-failure rendering 保留已提交的 `A/M/D`，并单独列出 failed file groups。

因此，当前 public contract 必须同时陈述两件事：

- intended authoring semantics；
- current runtime compatibility behavior。

## Compatibility Notes Boundary

overwrite-tolerant 行为属于 compatibility semantics，而不是推荐写法。

同样地：

- parser 若接受比 public guidance 更宽的 numbered old-side evidence，也不意味着这些写法自动进入默认教学面；
- unnumbered legacy authoring 若继续可解析，也不意味着 raw textual `@@` 仍应被教授为主要辅助定位路径。
- process-level environment 可以改变 default numbered-evidence mode；
- DocuTouch CLI 可以在单次 invocation 上显式 override 这一 mode，而 MCP surface 不新增对应参数。

接口层面的稳定表达应是：

- 事实性披露当前 runtime 会发生什么；
- 明确 preferred authoring pattern 仍然更窄；
- 不用正向示例把 overwrite-via-Add 或 overwrite-via-Move 教成技巧。

换言之：

- 可以披露 reality；
- 不能把它提升为 primary semantics。

## Success And Warning Contract

无 warning 的 success path 保持 core summary shape：

```text
Success. Updated the following files:
A path/to/file
M path/to/other
```

当成功路径触发 risky compatibility behavior 时，warning 以独立 diagnostic block 追加在 success summary 之后。

当前接口层面已接受的 warning family 包括：

- `ADD_REPLACED_EXISTING_FILE`
- `MOVE_REPLACED_EXISTING_DESTINATION`

warning contract 的稳定规则是：

- 只有行为实际发生时才触发 warning；
- warning 使用 machine-comprehensible、repair-oriented phrasing；
- warning block 不重写 primary success contract；
- help text 指回 preferred authoring alternative。

## Failure Surface

failure surface 继续遵守 repair-first contract。

接口层面可依赖的性质包括：

- partial success 时保留已提交路径的 accounting；
- failed file groups 被单独枚举；
- 诊断尽量指向最真实的 authored blame location；
- connected file groups 不发生半提交。

## Out-Of-Scope Here

本页不承担：

- 为什么 warning-first posture 更优的 decision rationale；
- path identity / Windows hardening plan；
- file-level implementation sequencing；
- future strict-profile debate；
- historical implementation status。

这些内容应分别交给 decisions、architecture、process assets 或 records。
