(knowledge-interfaces-read-file-sampled-view-spec)=
# Read File Sampled View Spec

## Role

本页记录 `read_file` sampled view 的 accepted contract surface。

它回答：

- sampled view 解决什么问题；
- public parameter shape 是什么；
- validation 与输出 contract 如何成立；
- 调用者应如何理解 sampled view 与 exact contiguous read 的边界。

## Interface Position

sampled view 是一种 dense local inspection mode。

它服务的认知活动是：

- 不需要重读全部正文；
- 但需要足够多、足够明确的局部证据来安全继续。

它不是：

- exact contiguous `line_range` 的替代品；
- arbitrary sampling DSL；
- whole-file summarizer；
- semantic shortcut for deep debugging or patch authoring。

## External Contract

Recommended external shape:

```text
read_file(
  relative_path: string,
  line_range?: range,
  show_line_numbers?: bool,
  sample_step?: positive integer,
  sample_lines?: positive integer
)
```

parameter semantics:

- `line_range`
  - 仍然定义 bounded region；
  - 仍然是 exact contiguous selector；
  - 推荐 public form 为 `start:stop`；
  - 允许省略端点，例如 `:50`、`50:`；
  - 允许负索引从文件尾部定位，例如 `-50:`、`:-1`；
  - 不支持 `step`；稀疏 inspection 仍由 `sample_step` / `sample_lines` 承担。
- `sample_step`
  - 每 N 行开始一个 sampled block；
  - 与 `sample_lines` 共同定义 sampled mode 的 cadence；
  - 仅当自身缺省时，默认基线值为 `5`。
- `sample_lines`
  - 每个 sampled block 呈现的连续行数。
  - 缺省时，默认基线值为 `2`；若调用者显式给出较小的 `sample_step`，则会下调为满足校验约束的最接近默认值。

## Activation And Defaulting Rule

- sampled mode 的 public activation surface 由 `sample_step` 与 `sample_lines` 组成。
- 调用者提供其中任一参数时，另一参数按默认基线补值后共同形成 sampled mode。
- sampled mode 下，缺省参数先补默认值，再进入 validation。
- 默认基线组合为：
  - `sample_step = 5`
  - `sample_lines = 2`
- 为避免默认补值把请求补成无效形态，还需要遵守两条自适应规则：
  - 若调用者省略 `sample_step`、但给出了更大的 `sample_lines`，则 effective `sample_step` 至少提升到 `sample_lines + 1`；
  - 若调用者给出了较小的 `sample_step`、但省略 `sample_lines`，则 effective `sample_lines` 会收缩到严格小于 `sample_step` 的最近默认值。

## Validation Rule

sampled mode 在完成默认补值后，其有意义形态为：

- `1 <= sample_lines < sample_step`

该规则保证 sampled mode 仍然是 sampled，而不是退化成近似连续阅读。

## Output Contract

### Content-First Output

sampled mode 不引入 out-of-band metadata header。

返回体仍然应是 content-first surface，而不是额外先打印：

- `sampled view`
- `range: ...`
- `sample_step: ...`
- `sample_lines: ...`

### Vertical Omission

vertical omission 使用单独一行：

```text
...
```

它表示显示窗口之间省略了整行内容。

## Reading Model

sampled view 的推荐理解是 confidence-oriented inspection。

它最适合：

- 最近刚写完的文件；
- 结构重复或规律性较强的文件；
- 继续推进前的低成本结构确认；
- 决定是否需要进一步 `read_file` 精读的快速检查。

它不适合：

- exact patch authoring；
- deep semantic debugging；
- subtle formatting 或 off-by-one investigation。

## Recommended Parameter Sets

Recommended set A: balanced local check

- `sample_step = 4`
- `sample_lines = 2`

Recommended set B: cheaper local check

- `sample_step = 5`
- `sample_lines = 2`

它是 ordinary post-write confidence check 的稳定推荐组合；partial sampled call 的默认基线只定义 sampled cadence。

Recommended set C: conservative local check

- `sample_step = 3`
- `sample_lines = 2`

## Prompt-Facing Guidance Boundary

工具描述应当把 sampled view 作为 dense local confidence check 来教：

- 对 bounded range 使用较小的 `sample_step`，例如 `3-5`；
- 对 exact contiguous read，优先使用 `line_range = start:stop` 这一路径；
- 当不知道总行数、但需要看尾部时，直接使用 `-N:` 这类 tail-relative form；
- `sample_lines = 2` 是稳定默认建议；
- 需要精确位置时显式开启 line numbers。

## Acceptance Boundary

本页只记录 sampled view 的 accepted interface contract。

它不承担：

- future whole-file skim preset；
- display-column accounting；
- richer omission rendering debate。
