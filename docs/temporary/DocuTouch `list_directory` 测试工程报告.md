# DocuTouch `list_directory` 测试工程报告

测试时间：2026-04-28  
测试范围：`/home/t103o/workbench` 当前工作区内，不跨盘、不扫描系统目录。  
测试对象：`mcp__docutouch__.list_directory`，附带一次 `set_workspace` 验证。  
结论先行：这个工具可用，而且很适合作为“大量文件阅读前的目录侦察工具”。它的优势是输出干净、文件边界稳定、带大小和行数、能遵守 hidden/gitignore/type 过滤；主要风险是 symlink 不显式标注、`max_depth=0` 的边界语义不直观，以及类型过滤时会保留一些目录上下文，初看可能误以为那些目录里已经列出了匹配文件。

## 1. 测试目标

本次不是只看“能不能列文件”，而是按测试工程的角度检查它能否稳定支撑真实开发工作流。核心问题有六个：

1. 正常目录能否被清晰列出，并提供文件大小、行数、目录/文件统计。
2. `max_depth` 是否能控制探索深度，并在截断处保留继续探索的目录上下文。
3. `show_hidden` 和 `include_gitignored` 是否独立生效，过滤统计是否可信。
4. `file_types` / `file_types_not` 是否符合 ripgrep 类型过滤直觉，错误 type 是否有清晰报错。
5. 对空目录、绝对路径、错误路径、文件路径、时间戳、符号链接等边界输入是否稳定。
6. 在较大子项目上是否仍能作为低噪声侦察入口，而不会一次性返回巨型正文。

## 2. 测试设计

我先把 workspace 设置为 `/home/t103o/workbench`，然后做两类测试：

第一类是当前仓库真实结构测试。对根目录和 `projects/codex` 这类较大子树做浅层列表，验证它在真实工作区中的输出质量、速度和噪声控制。

第二类是受控夹具测试。我临时创建了 `experiments/docutouch-listdir-fixture`，里面包含普通文件、Python 文件、JavaScript 文件、Markdown 文件、JSON/YAML 文件、深层目录、空目录、隐藏文件、隐藏目录、本地 `.gitignore`、被 gitignore 忽略的文件和目录。后面又临时加入文件 symlink 与目录 symlink，验证它对链接的处理。测试结束后，夹具已清理，`git status --short -- experiments/docutouch-listdir-fixture` 无输出，说明没有留下测试文件。

## 3. 正常路径与默认行为

默认调用 `list_directory(relative_path='.', max_depth=2)` 能正确列出工作区根目录。输出是树形结构，文件后面带大小和行数，例如 `README.md (809 B, 37 lines)`、`uv.lock (192.9 KB, 1543 lines)`。最后会汇总目录数和文件数，并报告被过滤条目数量，例如隐藏项、gitignored 项、两者兼具的项。

这个输出对模型阅读很友好：它没有把多个文件正文拼成一个巨大返回体，也不会像 `find` 那样只给路径而缺少阅读优先级信息。大小和行数能帮助判断下一步该读哪个文件，特别适合你 AGENTS 里说的“先使用目录类工具建立文件清单，再并行批量读取单文件”。

## 4. 深度控制

`max_depth=1` 时，只列出第一层目录和第一层文件，不展开子目录。受控夹具中表现为 `data/`、`empty_dir/`、`nested/`、`src/` 和 `README.md`。这很适合第一次粗扫。

`max_depth=3` 时，会展开到更深层，但对还没到叶子的深目录只保留目录节点。例如 `nested/deep/level/` 在深度不够时只作为继续探索线索存在。`max_depth=4` 时才显示 `nested/deep/level/file.txt`。这个行为符合“逐层侦察”的使用方式。

需要记录一个边界：传入 `max_depth=0` 没有报错，实际表现接近默认深度，而不是“只显示根目录”或“非法参数”。这不是致命问题，但语义不够严格。建议实际使用时避免传 0，明确使用 1、2、3 这类正整数。

## 5. 隐藏项与 gitignore

默认行为会隐藏 dotfile/dotdir，也会隐藏 gitignored 条目，并在末尾报告过滤数量。受控夹具默认输出中，隐藏项和 gitignored 项都没有出现在树里，末尾出现类似 `filtered: 6 entries (3 hidden, 3 gitignored, 0 both)`。

`show_hidden=true` 后，`.gitignore`、`.hidden_root.txt`、`.hidden_dir/note.md` 会显示出来，但 gitignored 的 `ignored.log`、`ignored_dir/`、`scratch.tmpignore` 仍然被过滤。

`include_gitignored=true` 后，gitignored 文件会显示，但隐藏项仍然隐藏。

`show_hidden=true` 与 `include_gitignored=true` 同时打开时，夹具内所有条目都会显示，末尾不再显示过滤统计。这个矩阵结果很清楚，说明两个开关是独立的，符合预期。

## 6. 文件类型过滤

`file_types=['python']` 能只保留 Python 文件，并保留到该文件的目录路径上下文。夹具中只显示 `src/main.py`。`file_types=['markdown']` 只显示 `README.md`，默认仍排除隐藏目录里的 `note.md`，这说明 type 过滤不会绕过 hidden/gitignore 规则。

`file_types_not=['markdown']` 能排除 Markdown 文件，同时保留其他普通文件。被 hidden/gitignore 排除的条目仍然不会显示。这里要注意：一旦启用 type filter，空目录一般不会被保留，输出更偏向“匹配文件的路径上下文”，而不是完整目录树。

未知类型，例如 `file_types=['definitely_not_a_real_rg_type']`，会返回清晰错误：`invalid file type filter: unrecognized file type`。这很好，因为它能快速暴露参数拼错，而不是静默返回空结果。

另一个边界：同时设置 `file_types=['python']` 和 `file_types_not=['python']` 时，结果没有文件，但仍保留了一些目录上下文。这符合“排除优先”的说明，但对用户来说可能略显意外。建议不要把同一类型同时放入 include 和 exclude。

## 7. 时间戳、绝对路径、空目录与错误输入

`timestamp_fields=['modified']` 会在文件项后追加 `modified=...`，`timestamp_fields=['created','modified']` 会同时显示创建和修改时间。当前观察到时间戳主要附在文件上，目录行不显示时间戳。时间格式包含时区和纳秒，适合诊断构建产物或最近修改文件。

绝对路径 `/home/t103o/workbench/experiments/docutouch-listdir-fixture` 可以正常工作，输出与相对路径等价。空目录会返回 `0 directories, 0 files`，这对判断目录是否真的为空很直接。

错误输入方面表现良好：不存在目录会报 `Directory does not exist`；把文件路径传给它会报 `Path is not a directory`；未知 type 会报参数错误。这些错误信息都具体到路径或参数，利于快速修正。

## 8. 符号链接行为

我临时创建了一个文件 symlink 和一个目录 symlink。结果显示：目录 symlink 会被当成目录展开；文件 symlink 会显示为普通文件，并且大小、行数按目标文件计算。工具没有在输出中标注这是 symlink。

这点是本次测试发现的最大注意事项。对于普通项目没问题；但如果仓库里有大量 symlink、vendor 链接、循环链接或跨目录链接，使用者需要先意识到它可能重复展开同一批文件，也可能让输出看起来像真实目录。报告级建议是：在已知 symlink 密集的仓库里，先用浅层深度观察，必要时结合 shell 的 `find -type l` 或 `rg --files` 做交叉确认。

## 9. 较大仓库表现

在 `projects/codex` 上测试浅层 type 过滤，工具返回速度仍然很快，输出不会读取文件正文，只列目录、文件、大小、行数和统计。`file_types=['rust']` 在 `max_depth=2` 下保留了大量 Rust crate 目录上下文，但没有把深层 Rust 文件全部展开出来。这个行为适合第一阶段定位模块，但不适合直接当作“找所有 Rust 文件”的最终清单。

如果目标是找具体文件，下一步应继续缩小到某个 crate 或子目录，再调用 `list_directory`。如果目标是全文搜索，则应该切换到 `search_text` 并限定类型，例如 C/C++ 仓库按你的规则必须限定 type，避免爆炸搜索。

## 10. 可用性评价

优点：
- 输出结构稳定，适合模型消费。
- 默认尊重 hidden 和 gitignore，噪声低。
- 文件大小与行数非常有价值，能帮助决定阅读顺序。
- 深度控制适合渐进侦察。
- 错误信息明确。
- 类型过滤复用 ripgrep 类型体系，和开发者习惯一致。
- 空目录、绝对路径、时间戳都能处理。

不足和风险：
- symlink 不标注，可能导致误判或重复探索。
- `max_depth=0` 语义不直观，建议避免使用。
- type 过滤时会保留目录上下文，不应把“目录出现”理解为“目录本身匹配类型”。
- 对超大仓库如果路径给得太宽，输出仍可能很长；它解决的是正文聚合风险，不等于可以无边界扫描。

## 11. 使用建议

我的建议是把 `list_directory` 作为默认第一步目录侦察工具使用，优先级高于手写 `find` 或 `ls -R`。推荐模式：

1. 先对任务相关根目录用 `max_depth=2` 或 `max_depth=3` 建立地图。
2. 看到候选子目录后，再对具体子目录继续调用，而不是一次拉满深度。
3. 需要看隐藏配置时临时加 `show_hidden=true`。
4. 需要看构建产物或缓存时才加 `include_gitignored=true`。
5. 面对语言仓库时使用 `file_types` 或 `file_types_not` 收敛范围。
6. 对 symlink 密集项目保持警惕，必要时交叉验证链接。
7. 大量阅读时继续遵守“并行批量单文件读取”，不要把正文聚合成巨型返回体。

## 12. 最终结论

`mcp__docutouch__.list_directory` 是好用的。它不是替代全文搜索的工具，也不是构建系统级文件索引；它最适合做代码阅读前的低成本、高信噪比目录侦察。在这个 workbench 里，它和你的工作规则高度匹配：先建清单、再分批读文件、保持范围收敛、避免一次性大文本拼接。

我会把它作为之后阅读仓库结构的默认工具之一。实际使用时我会注意三个边界：不要传 `max_depth=0`，不要把 type 过滤后的目录上下文误读成文件命中，不在 symlink 密集区域盲目展开。整体评价：推荐使用，适合作为常规工作流工具。