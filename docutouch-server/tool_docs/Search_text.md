`search_text` 是一个 ripgrep-compatible、LLM-friendly 的智能搜索工具。`query` 是要搜索的文本或模式；`path` 是搜索范围，可为单个 path、path 数组或 `pueue-log:<id>`。`rg_args` 只放 ripgrep flags/options；不要把搜索文本或路径放进 `rg_args`，例如只写 `--type py`，不要写 `--type py tests/tools`。工具会自动推断最合适的结果对象并尽量保持高信噪输出。

默认情况下，`search_text` 优先返回 DocuTouch 风格的 grouped 结果，适合 discovery 和后续 `read_file` 阅读。当 `rg_args` 表达了更明确的输出意图时，工具会自动切换到更合适的模式，例如：

- context flags (`-A/-B/-C`) -> `grouped_context`
- count flags (`-c`, `--count`, `--count-matches`) -> `counts`
- file-list flags (`-l`, `--files-with-matches`, `--files`) -> `files`
- `--json` -> `raw_json`
- 需要保留原生 rg 布局的组合 -> `raw_text`

`query_mode` 默认为 `auto`：如果查询在 regex 模式下无法编译，工具会自动回退为 literal 搜索，以降低作者输入摩擦。你也可以显式指定 `literal` 或 `regex`。`output_mode` 默认为 `auto`，但也可以强制指定为 `grouped`、`grouped_context`、`counts`、`files`、`raw_text` 或 `raw_json`。

把它理解成“一个统一的智能 `rg` 入口”：先优先用 `search_text`，而不是在工具和终端之间来回切换。
