# `search_text` Design Draft

> Note:
> This document remains useful as exploratory design background.
> The current implementation-facing UX contract now lives in
> `docs/search_text_ux_contract.md`.

## Purpose

`search_text` is not meant to replace raw `rg` in the terminal.
It exists to give LLM-oriented hosts a cleaner, more token-efficient search
surface over the same underlying ripgrep capability.

The terminal still remains the escape hatch for unrestricted raw search.
`search_text` exists for the common path where grouped, stable, low-noise
results are easier for an agent to consume than a flat `path:line:text` stream.

## Design Goal

The goal is not “more search power.”
The goal is:

- fewer repeated path tokens
- stable file boundaries
- small first-pass search results that naturally flow into `read_file`
- a contract that is simple enough to teach and hard to misuse

## Product Position

`search_text` should be understood as:

- a wrapped `rg`
- with grouped, LLM-friendly output shaping
- not a brand-new search engine
- not a giant CLI parameter mirror

The user and the model should both be able to understand:

- raw `rg` is still available in the terminal
- `search_text` is the cleaner structured surface

## Occam-Razor Contract

### Recommended input shape

```text
search_text(
  query: string,
  path: string,
  rg_args?: string
)
```

### Fields

- `query`
  - required
  - the search pattern passed to ripgrep

- `path`
  - required
  - may be relative or absolute
  - resolved exactly like the rest of DocuTouch path-taking tools
  - respects `set_workspace`, then `DOCUTOUCH_DEFAULT_WORKSPACE`, else errors

- `rg_args`
  - optional
  - raw advanced passthrough for uncommon cases
  - intended as an escape hatch, not the main teaching path
  - examples:
    - `-F`
    - `-i`
    - `-g '*.rs'`
    - `-C 2`
    - `-P '(?m)^foo'`

## Why this is the right minimal surface

This keeps the contract small and explicit:

- one required field
- one required path scope
- one raw escape hatch

It avoids:

- a 20-parameter schema that just mirrors ripgrep
- forcing the model to choose among many overlapping options
- inventing a fake regex dialect

It still preserves power because:

- the raw terminal remains available
- `rg_args` can pass through advanced ripgrep behavior when necessary

## Default Behavior

Even when `rg_args` is used, `search_text` should keep ownership of the output
shape.

That means the tool should:

- execute ripgrep
- parse the raw match output
- regroup the result by file
- emit a token-efficient grouped report

In other words:

- `rg_args` influences search behavior
- but not the fundamental output contract

## Output Shape

### Recommended default rendering

```text
search_text[summary]:
query: apply_patch
path: docutouch-server/src
files: 3
matched_lines: 8
matches: 8

[1] main.rs (4 lines, 4 matches)
  136 | "apply_patch",
  225 | async fn apply_patch_impl(&self, patch: String) -> Result<String, String> {
  391 | "apply_patch" => {
  392 | self.apply_patch_impl(

[2] stdio_smoke.rs (3 lines, 3 matches)
  69 | name: "apply_patch".into(),
  132 | name: "apply_patch".into(),
  584 | name: "apply_patch".into(),

[3] apply_patch.md (1 line, 1 match)
  1 | Apply a freeform patch. This is the primary structural editing tool.
```

## Key UX Rules

### 1. Group by file, not by raw match stream

Each file path should appear once.
This is the single biggest token win over raw `rg`.

### 2. Show an overview first

The tool should always surface:

- `query`
- effective path
- number of matched files
- number of matched lines

That lets the caller decide whether to continue with `read_file`.

### 3. Preserve file boundaries

This tool should point toward the next `read_file` call.
It should not try to become a replacement for file reading.

### 4. Avoid repeated path spam

Repeated full paths per line are low-value tokens for an LLM.
Grouping is the main optimization.

### 5. Do not add private truncation policy in v1

The host already applies hard output limits.
`search_text` should optimize shape, not invent a second hidden truncation
system.

That means:

- group by file
- emit a compact overview
- avoid repeated path spam
- but do not silently decide to drop a large tail of results inside the tool

## Simulated Examples

### Example A: one file, two matches

Input:

```text
query = "DOCUTOUCH_DEFAULT_WORKSPACE"
path = "."
```

Output:

```text
search_text[summary]:
query: DOCUTOUCH_DEFAULT_WORKSPACE
path: .
files: 2
matched_lines: 4
matches: 4

[1] docutouch-server/src/main.rs (3 lines, 3 matches)
  22 | const DEFAULT_WORKSPACE_ENV: &str = "DOCUTOUCH_DEFAULT_WORKSPACE";
  125 | workspace: Arc::new(RwLock::new(default_workspace_from_env())),
  838 | "Relative path requires workspace. Call set_workspace first, set DOCUTOUCH_DEFAULT_WORKSPACE, or use an absolute path."

[2] README.md (1 line, 1 match)
  79 | DOCUTOUCH_DEFAULT_WORKSPACE=/absolute/path/to/project cargo run -p docutouch-server
```

### Example B: many matches in one noisy file

Input:

```text
query = "apply_patch"
path = "."
```

Output:

```text
search_text[summary]:
query: apply_patch
path: .
files: 6
matched_lines: 41
matches: 41

[1] codex-apply-patch/src/lib.rs (18 lines, 18 matches)
  221 | pub fn apply_patch(
  289 | pub fn apply_patch_in_dir(patch: &str, cwd: &Path) -> Result<ApplyPatchReport, ApplyPatchError> {
  2139 | fn test_apply_patch_in_dir_reports_chunk_source_for_context_mismatch() {
  note: 15 more matches in this file

[2] docutouch-server/src/main.rs (9 lines, 9 matches)
  196 | async fn apply_patch_impl(&self, patch: String) -> Result<String, String> {
  391 | "apply_patch" => {
  21 | const APPLY_PATCH_TOOL_DESCRIPTION: &str = include_str!("../tool_docs/apply_patch.md");
  note: 6 more matches in this file

[3] docutouch-server/tests/stdio_smoke.rs (8 lines, 8 matches)
  71 | name: "apply_patch".into(),
  134 | name: "apply_patch".into(),
  589 | name: "apply_patch".into(),
  note: 5 more matches in this file
```

This demonstrates the intended grouped shape:

- file path once
- representative matches
- explicit note when a file has more hits than shown

### Example C: advanced ripgrep passthrough

Input:

```text
query = "warning\\["
path = "."
rg_args = "-P -g '*.rs'"
```

Output:

```text
search_text[summary]:
query: warning\[
path: .
rg_args: -P -g '*.rs'
files: 2
matched_lines: 5
matches: 5

[1] docutouch-server/src/main.rs (3 lines, 3 matches)
  299 | lines.push(format!("warning[{}]: {}", warning.code, warning.summary));
  748 | "error[OUTER_INVALID_ADD_LINE]: Add File block is malformed".to_string()
  752 | "error[OUTER_INVALID_LINE]: update hunk contains an invalid line".to_string()

[2] docutouch-core/src/patch_runtime.rs (2 lines, 2 matches)
  215 | fn outcome_warnings(warnings: &[OfficialApplyPatchWarning]) -> Vec<ApplyOutcomeWarning> {
  218 | .map(|warning| ApplyOutcomeWarning {
```

### Example D: union search with multiple hits on the same line

Input:

```text
query = "alpha|beta"
path = "src"
```

Output:

```text
search_text[summary]:
query: alpha|beta
path: src
files: 2
matched_lines: 4
matches: 6

[1] src/one.txt (2 lines, 4 matches)
  1 | alpha beta  [2 hits]
  3 | beta alpha  [2 hits]

[2] src/two.txt (2 lines, 2 matches)
  1 | beta
  2 | alpha
```

This is the expected v1 behavior:

- grouping is still by file
- line entries stay stable
- the summary reports both matched lines and total matches
- if one line contains multiple submatches, the line is shown once with a hit count

## What the tool should not do

- It should not invent its own regex dialect.
- It should not try to be smarter than ripgrep about searching.
- It should not dump giant concatenated file bodies.
- It should not mirror every ripgrep flag as a separate schema field in v1.
- It should not silently omit a large tail of results without saying so.

## Relationship to raw `rg`

This design assumes:

- power users and models can still use raw terminal `rg`
- `search_text` is the cleaned-up surface for the common case

That is a feature, not a contradiction.
The raw tool remains available for edge cases.
The wrapped surface exists to keep the common path cheaper and clearer.

## Suggested v1 Decision

If implemented, v1 should choose:

- tool name: `search_text`
- grouped-by-file output
- one required `query`
- one required `path`
- one optional raw `rg_args` escape hatch

This is the smallest contract that still feels powerful.
