# `list_directory` Literal Extension Filtering Design

## 1. Purpose

This document defines the replacement design for the `list_directory` filtering
surface. The design removes the previous ripgrep/ignore file-type filtering
contract and replaces it with a literal final-extension filtering contract.

The design applies to the `list_directory` MCP tool, the local CLI `list`
command, the shared core implementation, the rendered directory-tree surface,
and the matching tests and documentation.

The design is intentionally narrow. It defines one object: filtering directory
tree output by the final extension segment of file names. It does not define a
language taxonomy, a semantic file-type vocabulary, a MIME classifier, a
glob-pattern surface, or a future filename-pattern system.

## 2. Problem Statement

The previous `list_directory` surface exposed `file_types` and
`file_types_not` as ripgrep/ignore type filters. That contract was not aligned
with the role of `list_directory`.

`list_directory` is a low-cost directory reconnaissance tool. It helps an
agent establish a file map before reading larger amounts of context. Its
filtering surface should be mechanical, local, and directly checkable from file
names. Ripgrep type filters are a language-oriented grouping system. They map
names such as `rust`, `markdown`, or `yaml` to sets of extensions and file-name
patterns. That grouping system is useful for search tools, but it creates
unnecessary interpretation space in a directory-listing tool.

The old contract produced several avoidable failure modes:

- A caller could pass a natural word such as `text` and receive a hard error
  because no ripgrep type by that name existed.
- A caller could pass an extension-shaped value such as `yml` or `mdx` and
  expect direct extension matching, while the runtime interpreted the value as a
  ripgrep type name.
- A caller could pass a language group such as `rust` and receive `.rs` files,
  which made `file_types` a taxonomy surface rather than a literal file-name
  filter.
- The tool description had to explain ripgrep type aliases, custom type-add
  limitations, and type-list discovery. Those concerns do not belong in a
  directory reconnaissance contract.

The replacement design removes this ambiguity by deleting the type vocabulary
from `list_directory`. The caller supplies literal extension tokens. The
runtime compares those tokens to the final extension segment of each file name.

## 3. Design Principle

The design follows one governing principle:

> `list_directory` extension filtering is a literal file-name filter. The
> runtime must not infer language families, semantic file categories, or
> ripgrep type groups from caller-provided filter tokens.

The consequence is deliberate:

- `rs` matches files whose final extension is `rs`.
- `rust` matches files whose final extension is `rust`.
- `md` matches files whose final extension is `md`.
- `markdown` matches files whose final extension is `markdown`.
- `text` matches files whose final extension is `text`.
- `yaml` matches files whose final extension is `yaml`.
- `yml` matches files whose final extension is `yml`.

There is no conflict resolution between extension names and semantic names,
because the runtime has no semantic-name layer. Every valid token is an
extension token. A valid token that does not correspond to any file in the
target tree simply matches no files.

## 4. Object Model

### 4.1 Extension Token

An extension token is the caller-authored filter unit. It denotes one final file
extension segment.

An extension token has these properties:

- It is a single string.
- It is non-empty after trimming.
- It does not include a leading dot.
- It does not include another dot.
- It does not include a path separator.
- It does not include glob syntax.
- It does not denote a language, ecosystem, semantic category, MIME type, or
  ripgrep type.

The token `md` denotes the extension segment `md`. It does not denote the
Markdown family. The token `rs` denotes the extension segment `rs`. It does not
denote Rust as a language. The token `text` denotes the extension segment
`text`. It does not denote all textual files.

### 4.2 File Extension

A file extension is the final extension segment of a file name. It is the
segment after the last `.` in the file name, using the platform path parser's
ordinary file-extension interpretation.

Examples:

| File name | Extension used by this design |
| --- | --- |
| `main.rs` | `rs` |
| `script.py` | `py` |
| `README.md` | `md` |
| `notes.mdx` | `mdx` |
| `guide.rst` | `rst` |
| `config.yaml` | `yaml` |
| `config.yml` | `yml` |
| `Cargo.lock` | `lock` |
| `types.d.ts` | `ts` |
| `archive.tar.gz` | `gz` |
| `README` | no extension |
| `Makefile` | no extension |
| `.gitignore` | no extension |
| `.env` | no extension |

Only the final extension segment participates in this filter. Multi-part
suffixes such as `d.ts`, `test.ts`, or `tar.gz` are not part of this object.

### 4.3 Extension Filter

An extension filter is the pair of include and exclude extension-token sets.

The include set is provided by `file_extensions`. The exclude set is provided by
`file_extensions_not`.

The filter is inactive when both sets are empty. When the filter is inactive,
no file is removed by extension.

The filter is active when either set is non-empty. An active filter applies only
to files. It does not classify or match directories directly.

## 5. Public Interface

### 5.1 MCP Tool Arguments

The public MCP argument names are:

```text
file_extensions
file_extensions_not
```

The previous arguments are removed:

```text
file_types
file_types_not
```

The removed arguments must not remain as hidden compatibility aliases in the
MCP schema. The `ListDirectoryArgs` deserialization surface should reject
unknown fields so that stale calls using `file_types` or `file_types_not` fail
explicitly rather than silently running without the requested filter.

The public `list_directory` tool description should define extension filtering
without referencing ripgrep/ignore type aliases. A suitable tool description is:

```text
以 ASCII 树列出目录内容。默认显示文件大小与总行数，适合在大量阅读前建立文件清单。可选显示隐藏项、Git ignore 命中项和时间戳字段。`file_extensions` / `file_extensions_not` 按文件名最后一个扩展名做字面过滤，例如 `rs`、`py`、`md`、`mdx`、`rst`、`yaml`、`yml`、`json`、`toml`；不接受语言生态名、ripgrep type name、glob、路径或多段后缀。
```

The `file_extensions` parameter description should be:

```text
只显示指定后缀的文件。每个值是文件名最后一个 `.` 后的单段字面扩展名，不写前导点；例如 `rs` 匹配 `*.rs`，`py` 匹配 `*.py`，`md` 匹配 `*.md`，`mdx` 匹配 `*.mdx`，`yml` 只匹配 `*.yml`。不接受 `rust`、`markdown`、`text` 这类分组名；不接受 glob、路径或 `tar.gz` 这类多段后缀。默认不过滤。
```

The `file_extensions_not` parameter description should be:

```text
排除指定后缀的文件。值的解释规则与 `file_extensions` 相同；当 include 与 exclude 同时命中同一文件时，exclude 优先。默认不排除。
```

### 5.2 CLI Arguments

The CLI `list` command should expose extension terminology:

```text
list [path] [--max-depth N] [--show-hidden] [--include-gitignored] [-e|--extension EXT] [-E|--extension-not EXT] [--timestamp-field created|modified]
```

Accepted flag forms:

- `--extension EXT`
- `--extension=EXT`
- `--ext EXT`
- `--ext=EXT`
- `-e EXT`
- `-eEXT`
- `--extension-not EXT`
- `--extension-not=EXT`
- `--ext-not EXT`
- `--ext-not=EXT`
- `-E EXT`
- `-EEXT`

The old CLI flags are removed:

- `--type`
- `--type-not`
- `-t`
- `-T`

Passing an old flag should produce the existing unknown-flag error path. The
old flags should not be accepted as compatibility aliases because they preserve
the removed type-filter object.

The CLI help note should say:

```text
- `list` supports literal final-extension filters with `-e/--extension` and `-E/--extension-not`; pass `rs`, `py`, `md`, `yml`, not language names or globs.
```

## 6. Token Validation

### 6.1 Valid Token Shape

An extension token is valid when all of the following are true:

- The trimmed token is non-empty.
- The token does not start with `.`.
- The token does not contain `.`.
- The token does not contain `/` or `\`.
- The token does not contain NUL.
- The token does not contain whitespace.
- The token does not contain glob metacharacters: `*`, `?`, `[`, `]`, `{`, `}`.

The implementation may normalize ASCII case for comparison. If case
normalization is used, both caller tokens and file extensions should be
lowercased before set membership checks. Case normalization does not create a
vocabulary layer. It only prevents `.MD` and `md` from being treated as
different extension identities in a cross-platform directory listing surface.

### 6.2 Invalid Token Errors

Invalid token shape is an argument error. The runtime should return a direct
diagnostic that names the invalid field and value.

Examples:

| Input token | Error reason |
| --- | --- |
| `""` | empty extension token |
| `.md` | leading dot is not part of the token surface |
| `*.md` | glob syntax is not accepted |
| `tar.gz` | multi-part suffix is not accepted |
| `src/md` | path syntax is not accepted |
| `md rs` | whitespace is not accepted |

The error should guide the caller toward the valid surface:

```text
file_extensions contains invalid extension token '.md'; use 'md' without a leading dot
```

or:

```text
file_extensions contains invalid extension token 'tar.gz'; use only the final extension segment, such as 'gz'
```

### 6.3 No Unknown-Token Concept

The design has no unknown-token branch.

`banana` is a valid extension token. It denotes files whose final extension is
`banana`. If no such files exist under the target directory, the filtered tree
contains no matching files. That result is not an error and not a warning.

This rule is the central difference between the previous type-filter contract
and the new extension-filter contract. Type systems have unknown names.
Literal extension filters do not. A valid token is always meaningful because it
is itself the extension value being matched.

## 7. Filtering Semantics

The extension filter applies to file candidates after hidden and gitignore
eligibility have been evaluated.

For each file candidate:

1. Determine whether the file is excluded by hidden/gitignore rules.
2. If hidden/gitignore rules exclude the file, do not evaluate extension
   filtering for count attribution.
3. If extension filtering is inactive, keep the file.
4. If `file_extensions` is non-empty, keep the file only when it has a final
   extension and that extension is in the include set.
5. If `file_extensions_not` contains the file extension, exclude the file.
6. If include and exclude both match, exclude the file.

Files without an extension behave as follows:

- They are kept when extension filtering is inactive.
- They are filtered when `file_extensions` is non-empty.
- They are not affected by `file_extensions_not` because they have no extension
  to match.

Directories behave as follows:

- A directory is not directly matched by extension.
- A directory may be rendered as context when it contains rendered descendants.
- A directory may remain visible at a `max_depth` boundary according to the
  existing boundary-context behavior.

## 8. Display Surface

The tree output remains the primary surface. Extension filtering should not add
warning blocks for valid tokens that match no files.

The filter accounting should use extension terminology. The existing type count
should be renamed from `filtered_type_count` to `filtered_extension_count`.

Rendered summary example:

```text
project/
├── src/
│   └── main.rs (120 B, 6 lines)
1 directory, 1 file
filtered: 3 entries (0 hidden, 0 gitignored, 0 both, 3 extension)
```

The extension count represents entries filtered by extension after hidden and
gitignore filtering. Count categories should remain mutually attributable. An
entry already excluded as hidden or gitignored should not also be counted as
extension-filtered.

## 9. Implementation Plan

### 9.1 Core Layer

`docutouch-core/src/fs_tools.rs` should remove the dependency on
`ignore::types::{Types, TypesBuilder}` for directory extension filtering.
The `ignore` crate remains in use for gitignore matching.

`DirectoryListOptions` should change from:

```rust
pub file_types: Vec<String>,
pub file_types_not: Vec<String>,
```

to:

```rust
pub file_extensions: Vec<String>,
pub file_extensions_not: Vec<String>,
```

`DirectoryListResult` should change from:

```rust
pub filtered_type_count: usize,
```

to:

```rust
pub filtered_extension_count: usize,
```

The core should introduce an extension filter type:

```rust
struct ExtensionFilter {
    include: HashSet<String>,
    exclude: HashSet<String>,
}
```

The core should introduce these functions:

```rust
fn build_extension_filter(options: &DirectoryListOptions) -> std::io::Result<Option<ExtensionFilter>>;
fn parse_extension_token(field_name: &str, value: &str) -> std::io::Result<String>;
fn is_extension_filtered(path: &Path, filter: Option<&ExtensionFilter>) -> bool;
```

`walk_directory` and rendering logic should accept `Option<&ExtensionFilter>`
instead of `Option<&Types>`.

### 9.2 Server Layer

`docutouch-server/src/tool_service.rs` should change `ListDirectoryArgs` to use
the new field names and descriptions. It should also reject unknown fields for
this argument object.

The server implementation should pass `file_extensions` and
`file_extensions_not` into `DirectoryListOptions`.

The MCP tool description should be rewritten so that the public contract no
longer mentions ripgrep/ignore type names.

### 9.3 CLI Layer

`docutouch-server/src/cli.rs` should change `ListCommand` to hold extension
sets. The parser should remove `--type`, `--type-not`, `-t`, and `-T`.

The parser should add `--extension`, `--ext`, `-e`, `--extension-not`,
`--ext-not`, and `-E`.

The CLI usage and notes should use extension terminology.

### 9.4 Documentation Layer

Documentation should replace type-filter terminology with extension-filter
terminology:

- README table entry for `list_directory`
- README.zh-CN table entry for `list_directory`
- CLI guide if it mentions `--type`
- tool-surface guide if it describes file-type filtering
- any temporary report that is still used as current documentation

Historical records may remain historical if they are clearly archival. Current
public guidance should not teach the removed type-filter object.

## 10. Test Plan

### 10.1 Core Tests

The existing type-filter tests should be renamed and rewritten.

Required tests:

1. `list_directory_can_include_file_extensions`
   - Files: `src/main.rs`, `src/main.cpp`, `README.md`
   - Options: `file_extensions = ["rs"]`
   - Expected: `main.rs` appears; `main.cpp` and `README.md` do not.

2. `list_directory_can_exclude_file_extensions`
   - Files: `main.rs`, `README.md`
   - Options: `file_extensions_not = ["md"]`
   - Expected: `README.md` does not appear; `main.rs` appears.

3. `list_directory_extension_exclusion_wins_over_inclusion`
   - Files: `main.rs`, `README.md`
   - Options: include `rs`, `md`; exclude `rs`
   - Expected: `main.rs` does not appear; `README.md` appears.

4. `list_directory_keeps_max_depth_boundary_dirs_under_extension_filter`
   - Deep file: `src/nested/main.rs`
   - Options: `max_depth = 1`, `file_extensions = ["rs"]`
   - Expected: boundary directory context remains visible.

5. `list_directory_filters_last_extension_only`
   - Files: `types.d.ts`, `archive.tar.gz`, `schema.d.json`
   - Options: `file_extensions = ["ts"]`
   - Expected: `types.d.ts` appears; other files do not.

6. `list_directory_treats_arbitrary_word_as_literal_extension`
   - File: `note.banana`
   - Options: `file_extensions = ["banana"]`
   - Expected: `note.banana` appears.

7. `list_directory_does_not_match_extensionless_dotfiles`
   - Files: `.env`, `.gitignore`, `config.env`
   - Options: `file_extensions = ["env"]`
   - Expected: only `config.env` appears.

8. `list_directory_rejects_leading_dot_extension_token`
   - Options: `file_extensions = [".md"]`
   - Expected: invalid argument.

9. `list_directory_rejects_glob_extension_token`
   - Options: `file_extensions = ["*.md"]`
   - Expected: invalid argument.

10. `list_directory_rejects_multipart_extension_token`
    - Options: `file_extensions = ["tar.gz"]`
    - Expected: invalid argument.

11. `list_directory_reports_extension_filtered_count`
    - Options: include one extension while several visible files do not match.
    - Expected: display contains extension-filter accounting.

### 10.2 Server Tests

Required MCP tests:

1. `server_list_directory_can_filter_by_file_extension`
   - Arguments: `file_extensions = ["rs"]`
   - Expected: `.rs` file appears; non-`.rs` files do not.

2. `server_list_directory_can_exclude_file_extension`
   - Arguments: `file_extensions_not = ["md"]`
   - Expected: `.md` file does not appear.

3. `server_list_directory_rejects_removed_file_types_field`
   - Arguments: `file_types = ["rust"]`
   - Expected: invalid argument due to unknown field.

4. `server_list_directory_rejects_invalid_extension_token`
   - Arguments: `file_extensions = ["*.md"]`
   - Expected: invalid argument with extension-token guidance.

5. Tool schema test
   - Expected: `list_directory` schema contains `file_extensions` and
     `file_extensions_not`.
   - Expected: schema does not contain `file_types` or `file_types_not`.

### 10.3 CLI Tests

Required CLI tests:

1. `docutouch list --ext rs`
2. `docutouch list --extension rs`
3. `docutouch list -e rs`
4. `docutouch list --ext-not md`
5. `docutouch list --extension-not md`
6. `docutouch list -E md`
7. `docutouch list --type rust` fails as unknown flag.
8. `docutouch list --ext '*.md'` fails as invalid extension token.

## 11. Compatibility and Migration

This design intentionally breaks the previous type-filter contract.

The previous call:

```json
{"file_types": ["rust"]}
```

is no longer valid.

The replacement call is:

```json
{"file_extensions": ["rs"]}
```

The previous call:

```json
{"file_types": ["markdown"]}
```

does not become:

```json
{"file_extensions": ["markdown"]}
```

unless the caller specifically wants files ending in `.markdown`. For Markdown
files, the caller should pass the exact desired extensions:

```json
{"file_extensions": ["md", "mdx", "markdown"]}
```

The previous call:

```json
{"file_types": ["yaml"]}
```

becomes:

```json
{"file_extensions": ["yaml", "yml"]}
```

when both `.yaml` and `.yml` should be shown.

This migration is explicit. The runtime should not translate old type names to
new extension sets.

## 12. Non-Goals

The design explicitly excludes these objects:

- Ripgrep/ignore type aliases.
- Language ecosystem names.
- Semantic categories such as `text`, `docs`, or `config`.
- MIME detection.
- Content sniffing.
- Filename pattern matching.
- Glob matching.
- Multi-part suffix matching.
- Extensionless file matching.
- Dotfile-name matching.
- User-defined type-add rules.

Each excluded object may be valid in another tool or a future feature, but it
is not part of this artifact. Adding any of these objects would change the
problem definition and should require a separate design.

## 13. Evaluation Checklist

The implementation satisfies this design when all statements below are true:

- The MCP schema exposes `file_extensions` and `file_extensions_not`.
- The MCP schema does not expose `file_types` or `file_types_not`.
- Calls containing `file_types` fail rather than being silently ignored.
- `rs` matches `.rs`.
- `rust` matches `.rust`, not `.rs`.
- `md` matches `.md`.
- `markdown` matches `.markdown`, not `.md`.
- `text` matches `.text`, not `.txt`, `.md`, or `.rst`.
- `yaml` matches `.yaml`, not `.yml`.
- `yml` matches `.yml`, not `.yaml`.
- `tar.gz` is rejected as an invalid extension token.
- `*.md` is rejected as an invalid extension token.
- `.md` is rejected as an invalid extension token.
- `note.banana` is matchable with `file_extensions = ["banana"]`.
- `.env` is not matchable with `file_extensions = ["env"]`; `config.env` is.
- Extension filtering contributes to extension-filtered count.
- Existing hidden and gitignore filtering behavior remains intact.

## 14. Final Contract

`list_directory` provides directory-tree reconnaissance. Its extension filter
accepts literal final-extension tokens and compares them to the final extension
segment of file names. The filter has no vocabulary other than caller-authored
tokens. It does not interpret language names, semantic categories, ripgrep
types, glob patterns, paths, multi-part suffixes, dotfile names, or file
contents.

The public API uses `file_extensions` and `file_extensions_not`. The old
`file_types` and `file_types_not` fields are removed. Include and exclude sets
apply only to files; exclude wins over include; directories remain display
context rather than match subjects. Invalid token shape is an argument error.
Valid tokens that match no files produce an ordinary filtered directory result.

This contract keeps the tool mechanically simple, makes model calls easier to
audit, and removes the accidental complexity introduced by type vocabularies.
