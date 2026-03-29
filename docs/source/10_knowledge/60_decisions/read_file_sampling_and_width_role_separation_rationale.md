(knowledge-decisions-read-file-sampling-and-width-role-separation-rationale)=
# Read File Sampling And Width-Role Separation Rationale

## 作用域

本页记录 `read_file` 在 sampled-view activation 与 line-width control 之间采用职责分离的已接纳裁决理据。

它回答的是：

- 为什么 `sample_step` / `sample_lines` 与 `max_chars` 不应共享同一 activation 语义；
- 为什么 `line_range` 仍应保持 exact contiguous selector 的地位；
- 为什么 prompt-facing tool docs 应以正向职责定义为主，而不是暴露历史修复痕迹；
- 这一裁决对 interface、implementation 与 regression discipline 有什么后果。

## Decision

当前 accepted decision 为：

- `sample_step` 与 `sample_lines` 共同定义 `read_file` 的 sampled inspection cadence；
- `max_chars` 定义当前读取结果中每一条可见行的宽度约束；
- `line_range` 继续作为 bounded exact contiguous selector，而不是 sampled-mode hint；
- exact contiguous read 与 sampled inspection 是两种不同的读取 surface，不通过 display-width 参数相互折叠；
- prompt-facing tool description 与 parameter schema 应正向描述各参数职责，而不是通过历史问题或防御性反面提示来教学。

## Authority Basis

- `line_range` 的 contract 本体是“选中哪一段连续内容”，而不是“给渲染器一个大概意图”；
- `sample_step` / `sample_lines` 描述的是纵向观察 cadence，`max_chars` 描述的是横向显示宽度，它们属于两个不同的控制维度；
- 若把宽度参数并入 sampled activation，调用者会在 exact read 表面上得到 sampled surface，破坏最小惊讶原则；
- `read_file` 的 public surface 会直接暴露给模型，prompt-facing 文案若混入修复痕迹、反面警告或历史时间感，会污染模型对当前 contract 的学习；
- 因此，正确的稳定做法不是继续堆叠例外说明，而是把参数职责与 surface 边界在设计上拆清楚。

## Alternatives Considered

### Alternative A: 让任意“显示相关参数”共同触发 sampled mode

不接受。

理由：

- 这会把横向宽度控制误写成纵向采样信号；
- exact contiguous read 将不再由 selector 独立决定；
- 调用者需要额外记住隐藏 activation rule，而不是从参数名自然推导行为。

### Alternative B: 保留共享 activation，但在个别场景里额外特判

不接受。

理由：

- 这会把 contract 边界继续留在隐式分支里；
- 文档、实现与测试更容易再次漂移；
- “例外补丁”无法替代干净的参数角色划分。

### Alternative C: 继续靠 prompt-facing 反面提示来纠偏

不接受。

理由：

- 工具描述不是修复公告板；
- 对模型暴露“不会怎样”“别误解成什么”会引入不必要的注意力竞争；
- 若正确理解必须依赖反面提醒，说明 contract 结构本身还没有讲清楚。

## Accepted Consequences

- `read_file` interface spec 应把 sampled cadence 与 width control 分别定义；
- implementation 应让 exact contiguous read 与 sampled view 走各自清晰的渲染路径；
- regression test 必须覆盖“仅提供 `max_chars` 时仍保持 exact contiguous read”这条边界；
- prompt-facing tool docs、schema description 与 canonical interface page 应保持同一套正向职责语言；
- 若未来有人想把宽度控制重新并入 sampled activation，应先在 `docs/source/` 中提出新的 deliberation 或 decision，而不是直接改实现。

## Boundary

本页是 accepted rationale。

它不承担：

- `read_file` interface contract 正文；
- sampled view 的完整参数与输出格式细节；
- 具体 implementation status 或历史修复过程记录。

这些对象分别进入：

- {ref}`knowledge-interfaces-read-file-sampled-view-spec`
- `docutouch-core/src/fs_tools.rs`
- `docutouch-server/src/tool_service.rs`
- `30_records/60_status/`（若后续需要记录实施波次或审计状态）
