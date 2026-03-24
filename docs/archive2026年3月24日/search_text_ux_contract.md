# RFC Draft: `search_text` UX Contract

## Status

- Accepted for implementation
- Audience: DocuTouch maintainers, MCP host integrators, prompt/tooling authors

## 1. Summary

This document proposes a stable UX contract for `search_text`.

The design goal is not to create a new search engine.
The goal is to provide a wrapped ripgrep surface that is easier for LLM-oriented
hosts to consume in a `scan -> select -> inspect` workflow.

The tool should optimize:

- token efficiency
- stable file boundaries
- low-noise first-pass search output
- predictable follow-up into `read_file`

The tool should not:

- mirror the full ripgrep CLI surface
- replace raw terminal `rg`
- silently invent hidden search semantics
- collapse into a giant flat `path:line:text` stream by default

## 2. Product Position

`search_text` should be understood as a search index, not a file reader.

Its job is to answer:

- which files are likely worth reading next
- roughly how large the search result set is
- which matching lines best explain why a file is relevant
- what the next action should be

It is not responsible for returning full file bodies.
That responsibility belongs to `read_file`.

This yields a stable interaction loop:

1. `search_text` for discovery
2. `read_file` for inspection
3. `apply_patch` for modification

## 3. Cognitive and UX Framing

The recommended framing for maintainers and tool authors is:

- `Progressive Disclosure`
  `search_text` should expose an overview before exhaustive detail.
- `Information Scent`
  the result should make promising files easy to identify.
- `Cognitive Load Management`
  output should minimize repeated low-value tokens.
- `Signal-to-Noise Optimization`
  each rendered token should help route the next step.
- `Recognition over Recall`
  the caller should recognize likely-next files without reconstructing structure
  from a noisy flat stream.

These concepts are a better vocabulary than informal descriptions such as
"smart hiding" or "cute formatting".

## 4. External Contract

### 4.1 Proposed Tool Signature

```text
search_text(
  query: string,
  path: string | string[],
  rg_args?: string,
  view?: "preview" | "full"
)
```

### 4.2 Field Semantics

- `query`
  - required
  - the raw ripgrep search pattern
  - supports ordinary search, literal search, regex search, and union search

- `path`
  - required
  - search scope
  - may be a single path or an array of paths
  - each element may be a file path or directory path
  - multiple entries are treated as one union scope
  - resolved with the same workspace semantics as the rest of DocuTouch

- `rg_args`
  - optional
  - escape hatch for advanced search behavior
  - not the primary teaching path
  - must not be allowed to take ownership of render shape

- `view`
  - optional
  - defaults to `preview`
  - the only intended user-facing output-mode switch

### 4.3 Scope-Set Semantics

`path` remains the public field name for backward compatibility, but
semantically it should be understood as the search scope.

When `path` is an array:

- the effective scope is the union of all provided files and directories
- the caller can perform a second-pass search over a selected subset of
  previously discovered paths
- the tool still teaches one conceptual slot, rather than splitting the UX into
  separate `path` and `paths` fields

## 5. Two-View Model

The tool should expose exactly two views.

### 5.1 `preview`

`preview` is the default discovery mode.

It exists to support the first-pass question:

- what should I read next?

`preview` may render a subset of files and a subset of matching lines, but it
must do so explicitly and accountably.

It should report:

- total matched files
- total matched lines
- total matches
- rendered files
- rendered lines
- omitted files or lines when omission occurs

This is not a hidden truncation policy.
This is an explicit overview representation.

### 5.2 `full`

`full` is the exhaustive grouped mode.

It exists to answer:

- I already know this query is relevant; now show me the full grouped result set

`full` should preserve the same grouped-by-file shape as `preview`, but it
should not intentionally omit matched files or matched lines.

`full` is still not raw `rg`.
It remains a DocuTouch-shaped, grouped representation.

## 6. Output Contract

### 6.1 Common Shape

Both views should preserve the same high-level structure:

```text
search_text[<view>]:
query: ...
scope: ...
files: ...
matched_lines: ...
matches: ...
...

[1] file/path (...) 
  <line> | <rendered snippet>
  ...
```

The view should change how much is rendered, not the overall reading model.

### 6.2 Header Fields

Required header fields:

- `query`
- `scope`
- `files`
- `matched_lines`
- `matches`

Conditional header fields:

- `rg_args`
  only when non-empty
- `rendered_files`
  required in `preview` when omission is possible
- `rendered_lines`
  required in `preview` when omission is possible

### 6.3 Scope Normalization

The output should prefer `scope` over raw input echoing.

Rationale:

- raw input may be absolute while grouped file paths are workspace-relative
- mixed coordinate systems increase reading friction
- `scope` is a better product term than `path` for rendered output

When multiple paths are provided, the rendered output should still preserve the
singular product vocabulary `scope`.

Acceptable rendering examples:

- `scope: src`
- `scope: [src, tests]`
- `scope: 3 roots`

## 7. Ranking and File Ordering

Default ordering should optimize decision value, not implementation convenience.

Recommended stable sort:

1. `matched_lines` descending
2. `matches` descending
3. `path` ascending

Rationale:

- dense files are usually better first-pass candidates
- deterministic output still matters
- alphabetical ordering alone is low-value for attention allocation

## 8. Snippet Rendering

### 8.1 `preview`

`preview` should render representative matching lines only.

The output should favor:

- one path per file block
- a small number of line entries per file
- explicit notes when more matched lines exist

Recommended note shape:

```text
note: 15 more matched lines in this file
```

### 8.2 Long-Line Treatment

`preview` should be allowed to render snippet windows rather than raw full line
text for very long lines.

Preferred rule:

- use ripgrep submatch spans
- render a bounded window around the span
- add `...` only where line text is clipped

This is presentation shaping, not search-result loss.

### 8.3 `full`

`full` should prefer showing every matched line in grouped form.

Whether extremely long lines are still windowed in `full` is an implementation
decision, but if any clipping occurs it must be explicit in the rendered text.

## 9. `rg_args` Taxonomy

The tool must clearly distinguish two categories of ripgrep flags.

### 9.1 Search-Behavior Flags

These affect matching semantics or search scope and may be accepted via
`rg_args`.

Examples:

- `-F`
- `-i`
- `-g '*.rs'`
- `-P`
- `--max-count`

### 9.2 Render-Shaping Flags

These affect output shape and must remain reserved by `search_text`.

Recommended canonical term:

- `render-shaping flags`

Alternative acceptable term:

- `output-shaping flags`

The important property is that the term is consistent across:

- tool descriptions
- error messages
- prompt-facing injected documentation
- maintainer docs

Recommended rejected set:

- `--json`
- `--heading`
- `--no-heading`
- `--color`
- `-n`
- `--line-number`
- `-N`
- `--no-line-number`
- `-c`
- `--count`
- `--count-matches`
- `-l`
- `--files-with-matches`
- `--files-without-match`
- `--files`
- `--type-list`
- `--replace`
- `-A`
- `-B`
- `-C`
- `--context`
- `--before-context`
- `--after-context`

Rationale:

- the tool already owns grouped rendering
- the tool already owns line-number rendering
- context lines are a presentation-layer concern unless explicitly modeled by the
  tool itself
- partially passing through render-shaping flags creates contract ambiguity

## 10. Why `-n` Must Be Rejected

`-n` must be rejected for more than one reason.

### 10.1 It is redundant

`search_text` already runs ripgrep with line numbers and uses those numbers in
its grouped rendering.

### 10.2 It weakens ownership boundaries

If `-n` is accepted, it becomes harder to explain why other render-shaping
flags are rejected.

### 10.3 It teaches the wrong mental model

The caller should learn that:

- `rg_args` changes how the search is performed
- `search_text` changes how the result is rendered

This boundary is the core product contract.

## 11. Why Context Flags Must Also Be Rejected Unless Modeled

Flags such as `-C`, `-A`, and `-B` are easy to misclassify.

They look like search modifiers, but in practice they are render-shaping flags
because they ask for additional context lines around a match.

If the tool does not parse and render those context lines deliberately, then
accepting the flags creates a false affordance.

That is worse than a clean rejection.

Recommended rule:

- reject these flags until the tool introduces first-class context rendering

## 12. User Stories

### 12.1 Story A: Narrow ordinary search

Goal:

- find the implementation of `search_text_impl`

Call:

```text
search_text(
  query: "search_text_impl",
  path: "prototype_development/docutouch/rust/docutouch-server/src",
  view: "preview"
)
```

Expected behavior:

- return one or two file blocks
- identify the implementation line immediately
- point naturally toward `read_file`

### 12.2 Story B: Broad ordinary search

Goal:

- find the most relevant `apply_patch` implementation files across the Rust
  workspace

Call:

```text
search_text(
  query: "apply_patch",
  path: "prototype_development/docutouch/rust",
  view: "preview"
)
```

Expected behavior:

- rank dense source files before incidental docs
- show only representative entries in `preview`
- make omission explicit

### 12.3 Story C: Exhaustive grouped follow-up

Goal:

- inspect all grouped matches once the query is known to be relevant

Call:

```text
search_text(
  query: "apply_patch",
  path: "prototype_development/docutouch/rust",
  view: "full"
)
```

Expected behavior:

- preserve grouped-by-file representation
- return the full grouped match set

### 12.3.1 Story C2: Progressive narrowing with multiple paths

Goal:

- run a second-pass search across a selected subset of files or directories

Call:

```text
search_text(
  query: "workspace",
  path: [
    "prototype_development/docutouch/rust/docutouch-server/src/main.rs",
    "prototype_development/docutouch/rust/README.md",
    "prototype_development/docutouch/rust/docs/search_text_ux_contract.md"
  ],
  view: "preview"
)
```

Expected behavior:

- treat the provided array as one union scope
- support precise follow-up discovery without forcing the caller to create a
  temporary directory boundary
- preserve the same output contract as single-path search

### 12.4 Story D: Union search

Goal:

- search `alpha|beta` as a single regex query

Call:

```text
search_text(
  query: "alpha|beta",
  path: "src",
  view: "preview"
)
```

Expected behavior:

- keep both `matched_lines` and `matches`
- annotate same-line multi-hit cases with hit counts

### 12.5 Story E: Literal exact search

Goal:

- search a literal string with regex metacharacters

Call:

```text
search_text(
  query: "warning[",
  path: ".",
  rg_args: "-F -g '*.rs'",
  view: "preview"
)
```

Expected behavior:

- treat the query literally because `-F` is a search-behavior flag
- keep grouped rendering fully owned by `search_text`

## 13. Prompt-Facing Documentation Guidance

Because tool descriptions are often injected into system or tool prompts, the
tool documentation should teach the contract before the first error occurs.

Recommended guidance for the tool description:

- say that `search_text` is a grouped ripgrep wrapper for the common case
- say that `rg_args` is only for search-behavior flags
- say that render-shaping flags are reserved by `search_text`
- say that raw terminal `rg` remains available for unrestricted usage
- say that `preview` is for overview and `full` is for exhaustive grouped output

Suggested wording fragment:

```text
`rg_args` is an escape hatch for advanced search-behavior flags such as `-F`,
`-i`, `-g`, and `-P`. Render-shaping flags such as `--json`, `-n`, `-c`, `-l`,
and context flags (`-A/-B/-C`) are reserved by `search_text` because the tool
owns grouped rendering.
```

## 14. Non-Goals

This proposal intentionally does not try to solve:

- giant-codebase defensive limits at the tool layer
- unrestricted raw-search workflows
- every possible ripgrep customization
- semantic ranking heuristics based on deep code understanding

Those concerns belong either to:

- host orchestration rules
- user discipline
- future specialized tools

## 15. Implementation Notes

This RFC intentionally focuses on external UX, not internal constants.

Examples of implementation choices that should remain internal:

- how many files `preview` renders by default
- how many lines per file are shown in `preview`
- exact snippet window width
- exact tie-break details after the documented sort contract

These can evolve without changing the externally taught mental model.

## 16. Recommended Next Step

The next implementation step should prioritize:

1. introducing `view = preview | full`
2. formally adopting the `render-shaping flags` taxonomy
3. rejecting context flags until they are truly modeled
4. changing default ordering from path-first to relevance-first deterministic
   ordering
5. making `preview` omission explicit rather than accidental
