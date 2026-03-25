(knowledge-decisions-apply-patch-numbered-anchor-guidance-rationale)=
# Apply Patch Numbered-Anchor Guidance Rationale

## 作用域

本页记录 `apply_patch` line-number-assisted locking 在 prompt-facing guidance 上的已接纳裁决理据。

它回答的是：

- 为什么 `@@` raw text header 不应继续被教授为主要辅助定位手段；
- 为什么 public guidance 应收敛到 single numbered anchor form；
- 为什么 parser support 与 public recommendation 必须继续分层；
- 这一裁决对后续 spec、implementation 与 tool docs 有什么后果。

## Decision

当前 accepted decision 为：

- `apply_patch` 的 public recommended auxiliary-location form 收敛为 single numbered anchor；
- canonical prompt-facing shape 为 `@@ N | visible text`；
- numbered evidence 的默认解释模式为 `header_only`，即默认只解释 numbered `@@` header；
- body-level dense numbered old-side evidence 只作为 advanced mode 存在，不进入默认 public guidance；
- advanced mode 由 environment / CLI override 控制，而不是进入 MCP tool schema；
- 不再把 raw textual `@@ def handler():` / `@@ class Example` 教成主要辅助定位路径；
- parser support 可以保留或引入更密的 numbered old-side evidence，但这些 form 不进入默认示例面。

## Authority Basis

- 当前 `@@` runtime 语义只是单行 coarse pre-anchor，不足以单独承担强定位；
- raw textual anchor 只能缩窄搜索窗口，不能给出全局唯一的 old-side 锁定；
- absolute 1-indexed line number 与 visible text 组合后，才形成真正有判别力的辅助定位证据；
- 大多数 patch 不需要把整段 old-side 全部编号，一个 single numbered anchor 已足够覆盖主路径；
- dense body-level numbering 虽然对少数高歧义场景有价值，但也会与原文本本来就长成 `N | text` 的内容发生解释竞争；
- 因此 body-level numbered evidence 更适合作为 advanced capability 保留，而不是默认解释器的常态；
- public tool docs 的职责是给出 ROI 最高、最自然、最稳定的 canonical authored shape，而不是枚举 parser 的全部容忍面。

## Alternatives Considered

### Alternative A: 继续把 raw textual `@@` 教成主要辅助定位路径

不接受。

理由：

- 这会继续把弱 pre-anchor 误写成强定位工具；
- 它无法解释“为什么这一行在全文件中足够唯一”；
- 这会让 prompt-facing guidance 再次高估 `@@` 的真实权能。

### Alternative B: 把 maximal numbered old-side evidence 作为默认 public form

不接受。

理由：

- 这会把少数高歧义场景抬升为默认 authored burden；
- 主路径会平白增加 token 与书写负担；
- 这会错误暗示 ordinary patch body 本身几乎没有定位价值。

### Alternative C: 继续投资 stacked multiple `@@` hierarchy

不接受。

理由：

- 它不是当前 parser/runtime 的可信延长线；
- 它把 `@@` 误写成 scope system，而不是 old-side evidence entry；
- line-number-assisted locking 已经提供了更直接、更可验证的增强方向。

## Accepted Consequences

- candidate spec 应明确区分 parser support set 与 prompt-preferred subset；
- tool docs 应把 `@@ N | visible text` 写成 canonical teaching form；
- default runtime mode 应保持 `header_only`，而不是默认把 dense numbered old-side evidence 全面上浮；
- CLI 可提供单次 invocation override，environment 可提供 process-level default；
- MCP 不新增专门的 numbered-evidence 参数，而是只受 server process environment 影响；
- implementation 应把 line-number assist 设计为 old-side evidence strengthening，而不是新工具身份；
- runtime 应以 original-snapshot 语义解释 numbered old-side evidence，而不是采用 intermediate-state-dependent 解释；
- 若 parser 支持比 public guidance 更宽，文档必须诚实说明“可解析”与“推荐写法”不是同一层对象。

## Boundary

本页是 accepted rationale。

它不承担：

- line-number-assisted grammar 的完整形式化正文；
- implementation phase sequencing；
- actual rollout status。

这些对象分别进入：

- `20_deliberation/40_candidate_specs/`
- `15_process_assets/10_exec_plans/`
- `30_records/60_status/`
