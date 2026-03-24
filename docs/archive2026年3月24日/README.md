# DocuTouch Docs Index

本目录用于沉淀 DocuTouch Rust 工作区的长期设计信息、维护规则与阶段规划。

这些内容的目标不是替代源码注释，也不是替代即时聊天记录，而是把未来维护最容易丢失的上下文固化下来，避免项目方向在数周或数月后重新漂移。

## 文档清单

- `product_positioning.md`
  项目定位、产品边界、为什么它是面向大模型的基础工具层、为什么当前优先 MCP/注入式而不是把 CLI 作为主形态。

- `maintainer_guide.md`
  维护者手册。包括修改原则、决策记录约定、何时更新文档、测试与回归要求、上游同步边界、如何避免项目逐渐失焦。

- `roadmap.md`
  未来阶段安排。区分近期工作、中期工作与暂缓事项，避免“想到什么加什么”的堆叠式演化。

- `apply_patch_semantics_plan.md`
  `apply_patch` 的语义、UX、warning、路径同一性与 Windows 边界硬化计划记录。

- `apply_patch_diagnostics_spec.md`
  `apply_patch` 诊断系统的详细设计规格。重点关注 source-span grade execution diagnostics、warning/error 统一语法、inline self-contained repair contract，以及宿主审计边界应如何与工具职责分离。

- `diagnostics_polish_spec.md`
  diagnostics 打磨的正式判断规格。重点关注哪些 polishing 值得继续做、哪些应明确停止、局部极限长什么样，以及如何用统一标尺审判新的优化提议。

- `cli_adapter_spec.md`
  Rust CLI 适配器的详细设计规格。重点关注 shared semantics 抽取、CWD 作为隐式 workspace anchor、MCP/CLI parity，以及如何避免复制出第二套产品语义。

- `read_file_sampled_view_spec.md`
  `read_file` 采样检查视图的详细设计规格。重点关注 confidence-oriented inspection、`sample_step / sample_lines / max_chars`、以及纵向与横向省略的显式语义。

- `apply_splice_spec.md`
  `apply_splice` 的正式产品/设计契约。重点关注独立工具身份、面向现有片段的结构操作边界、由八个 transfer 动作加一个 `Delete Span` 原语构成的完整动作基、基于绝对行号的选择语义，以及与 `apply_patch` 的能力边界。

- `temporary/为什么 \`apply_splice\` 与 \`apply_patch\` 必须分立.md`
  关于两类工具为何必须分立的论证文。它不是实现计划，而是对象边界与产品身份的论证材料；当讨论“是否应折叠为一个工具”时，应先读此文再进入实现层争论。

- `ux_hardening_plan.md`
  把当前 UX 问题、已完成项、剩余问题与实施波次整理成统一计划，避免后续只修局部而失去整体顺序。

- `temporary/diagnostics_dx_repair_program.md`
  当前 diagnostics DX 修复工程的母计划。用于协调文档审计、代码收敛、测试加固与收尾验收；在本轮工作中应优先于旧的临时 diagnostics 实施计划读取。

- `temporary/engineering_quality_wave_20260323/plan.md`
  当前工程质量波次的执行计划。重点关注 shared patch adapter、provenance ownership、smoke harness hardening，以及稳定文档与实现现状的对齐收口。

- `../docutouch-server/tool_docs/read_files.md`
  `read_files` 的退役记录。解释它为什么存在、为什么移除，以及为什么当前推荐重复调用普通 `read_file`。

- `search_text_design.md`
  基于 ripgrep 的搜索包装设计草案。重点不是新增搜索能力，而是把高频文本搜索结果整理成对大模型更友好的分组形态，降低重复路径噪音与 token 浪费。

- `search_text_ux_contract.md`
  `search_text` 的当前正式 UX 契约。定义两种视图、scope 语义、`rg_args` 分类、render-shaping flags 规则，以及 prompt-facing 描述建议。

## 使用建议

- 第一次接手本工作区时，先读 `product_positioning.md`。
- 准备做较大改动时，先读 `maintainer_guide.md`。
- 讨论是否要扩功能、改边界、做新接口时，先看 `roadmap.md`。
- 讨论 `apply_patch` 的行为、兼容性、warning 或路径语义时，直接看 `apply_patch_semantics_plan.md`。
- 讨论 `apply_patch` 诊断系统下一步应该怎么做、理想诊断长什么样时，优先看 `apply_patch_diagnostics_spec.md`。
- 讨论 Rust CLI 适配器应该如何设计、如何受控推进、如何与 MCP 保持等量语义时，优先看 `cli_adapter_spec.md`。
- 讨论 `read_file` 的采样检查视图应该如何设计、如何避免歧义、如何给出推荐参数组合时，优先看 `read_file_sampled_view_spec.md`。
- 讨论 `apply_splice` 的正式契约、完整动作基、窄语义边界，以及 source/target 选择契约时，优先看 `apply_splice_spec.md`。
- 讨论 `apply_splice` 与 `apply_patch` 为什么必须分立、为什么不应以“都能改文件”为由合并时，先看 `temporary/为什么 \`apply_splice\` 与 \`apply_patch\` 必须分立.md`。
- 讨论整体 UX 收敛顺序、剩余问题盘点和实施波次时，直接看 `ux_hardening_plan.md`。
- 讨论是否要新增 `search_text` 以及它应该长什么样时，直接看 `search_text_design.md`。
- 讨论 `search_text` 的当前对外契约、工具提示文案和实现收口边界时，优先看 `search_text_ux_contract.md`。

## 更新原则

- 文档要记录“为什么这样做”，不只记录“做了什么”。
- 当源码行为、产品立场、优先级排序发生变化时，必须同步更新本目录。
- 临时聊天结论如果影响未来维护，应尽快落入本目录，而不是留在聊天记录里。
