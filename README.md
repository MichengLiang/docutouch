# DocuTouch

[![release](https://github.com/MichengLiang/docutouch/actions/workflows/release.yml/badge.svg)](https://github.com/MichengLiang/docutouch/actions/workflows/release.yml)
[![npm publish](https://github.com/MichengLiang/docutouch/actions/workflows/npm-publish.yml/badge.svg)](https://github.com/MichengLiang/docutouch/actions/workflows/npm-publish.yml)
[![GitHub release](https://img.shields.io/github/v/release/MichengLiang/docutouch?sort=semver)](https://github.com/MichengLiang/docutouch/releases)
[![npm](https://img.shields.io/npm/v/docutouch)](https://www.npmjs.com/package/docutouch)
[![license](https://img.shields.io/github/license/MichengLiang/docutouch)](LICENSE)

Language:

- [English](README.md)
- [Simplified Chinese](README.zh-CN.md)

DocuTouch is a structural file-tooling workspace for coding-agent workflows. It provides an MCP server, a local CLI, and a Rust core for reading, searching, inspecting, and editing files through authored program surfaces instead of natural-language editing requests.

The project is built around one practical claim: when a model changes files, the operation should preserve stable evidence, file boundaries, repair accounting, and replayable input. DocuTouch therefore treats file context, selections, patch groups, AST matches, and background logs as explicit objects.

## What Ships

| Surface | What it is | Primary user |
| --- | --- | --- |
| `docutouch` binary | Stdio MCP server and local CLI from `docutouch-server` | Agents, local operators, MCP hosts |
| `docutouch-core` | Shared Rust library for filesystem tools, search surfaces, rewrite/splice runtimes, patch presentation, and structural search | Maintainers and downstream Rust integration |
| `codex-apply-patch` | Vendored fork of OpenAI Codex `apply-patch` with DocuTouch runtime semantics | Patch runtime and lineage audits |
| `npm/` package | Thin Node launcher that downloads the matching GitHub Release binary | Users who want `npx docutouch` or global npm install |
| `.github/workflows/release.yml` | Tag-driven GitHub Release build for Windows x64 and Linux x64 binaries | Release maintainers |
| `.github/workflows/npm-publish.yml` | npm trusted-publishing workflow with provenance | Release maintainers |

DocuTouch is not a second implementation of Git, ripgrep, ast-grep, or Pueue. It wraps those tools where appropriate and exposes outputs that are easier for coding agents to consume and retry.

## When To Use It

Reach for DocuTouch when a coding agent needs to:

- build a file list before reading large amounts of context
- read one file or one log handle with stable line ranges and optional line numbers
- search code with ripgrep while keeping grouped, file-oriented results
- run AST-aware searches through `ast-grep` and reopen the exact match later
- apply a structured patch and keep successful independent file groups committed
- replace or delete a selected old span using line-number evidence
- copy, move, delete, or replace already-existing text spans without restating the whole body
- wait for Pueue tasks and pass returned log handles back into `read_file` or `search_text`

DocuTouch is especially useful in repair loops. A failed operation should leave enough structured evidence for the next attempt to modify only the failed part.

## Tool Inventory

| MCP tool | Object boundary | Notes |
| --- | --- | --- |
| `set_workspace` | Default workspace for relative paths | Stores one canonical base path for the current MCP service instance. |
| `list_directory` | Directory tree | Renders an ASCII tree with optional hidden files, gitignored files, timestamps, and ripgrep/ignore file-type filters. |
| `read_file` | One file or one `pueue-log:<id>` handle | Supports `start:stop` line ranges, negative tail-relative bounds, line numbers, sampled inspection, and horizontal clipping. |
| `search_text` | Ripgrep-compatible text search | Accepts one path, many paths, or `pueue-log:<id>`; infers grouped, context, counts, files, raw text, or raw JSON output from `rg_args`. |
| `structural_search` | AST search session | Runs `ast-grep` pattern/rule queries and registers result groups for `expand`, `around`, and `explain_ast`. |
| `wait_pueue` | Pueue wait snapshot | Waits for explicit task ids or the current unfinished snapshot and returns reusable `pueue-log:<id>` handles. |
| `apply_patch` | Patch-shaped file mutation program | Supports Add/Delete/Update/Move file operations, connected file groups, partial success across independent groups, warning blocks, and repair artifacts. |
| `apply_rewrite` | Numbered-selection-locked rewrite program | Selects existing old spans by line number plus visible text, then deletes or replaces them with authored text. |
| `apply_splice` | Existing-span transfer program | Copies, moves, deletes, inserts, appends, or replaces spans that already exist in files. |

The local CLI covers the same operational family for `list`, `read`, `search`, `wait-pueue`, `patch`, `rewrite`, and `splice`. `set_workspace` and `structural_search` are MCP-facing tools.

## Editing Model

DocuTouch keeps three editing tools separate because they express different objects.

| Tool | Use it when | Old-side evidence | New text authoring |
| --- | --- | --- | --- |
| `apply_patch` | You are changing files into a new text state. | Patch context, optional numbered `@@` header anchor. | Authored through patch hunks or added files. |
| `apply_rewrite` | You are replacing or deleting specific existing spans and want selection-locked evidence. | Numbered selection lines plus optional `... lines omitted ...`. | Authored literally inside `*** With ... *** End With`. |
| `apply_splice` | You are relocating, duplicating, deleting, or replacing text that already exists. | Numbered source and target selections. | None inside the splice program; transferred text comes from the selected source span. |

This separation matters for agent reliability. Patch is best for producing new text states. Rewrite is best when the old span should be explicitly locked before replacement. Splice is best when the desired content is already in the repository and should be moved or reused without reauthoring.

## `apply_patch`

`apply_patch` is the patch-shaped structural write tool. It accepts the familiar patch envelope:

```text
*** Begin Patch
*** Update File: src/app.py
@@
-print("Hi")
+print("Hello")
*** End Patch
```

Supported file operations are:

- `*** Add File: <path>`
- `*** Delete File: <path>`
- `*** Update File: <path>`
- `*** Move to: <new path>` after an update header

When ordinary context is not unique enough, one numbered anchor can be attached to a hunk header:

```text
@@ 120 | def handler():
```

The default numbered-evidence mode is `header_only`: numbered `@@` headers are interpreted, while body text that happens to look like `121 | value = 1` remains ordinary patch text. The advanced `full` mode is available for dense old-side numbered evidence when an operator explicitly enables it.

The DocuTouch runtime adds behavior around the upstream patch shape:

- connected file groups commit atomically
- disjoint file groups can produce `PartialSuccess`
- successful runs can include warning blocks without becoming failures
- failed patch files can be persisted under `.docutouch/failed-patches/`
- diagnostics include committed changes, failed groups, attempted changes, and repair hints

A warning is rendered as a code-bearing block:

```text
Success. Updated the following files:
M notes.md

warning[ADD_REPLACED_EXISTING_FILE]: Add File targeted an existing file and replaced its contents
  --> notes.md
  = help: prefer Update File when editing an existing file
```

A partial failure starts with a stable error code:

```text
error[PARTIAL_UNIT_FAILURE]: patch partially applied
```

The output then separates what landed from what still needs repair.

See also:

- [codex-apply-patch/README.md](codex-apply-patch/README.md)
- [codex-apply-patch/UPSTREAM_LINEAGE.md](codex-apply-patch/UPSTREAM_LINEAGE.md)
- [codex-apply-patch/LOCAL_DIVERGENCES.md](codex-apply-patch/LOCAL_DIVERGENCES.md)

## `apply_rewrite`

`apply_rewrite` is a rewrite-program tool for selection-locked edits. Each rewrite action starts by selecting an existing contiguous span with absolute line numbers and full visible line content. The action then either deletes that span or replaces it with literal authored text.

A minimal line replacement looks like this:

```text
*** Begin Rewrite
*** Update File: src/app.py
@@ replace the selected print line
12 | print("Hi")
*** With
print("Hello")
*** End With
*** End Rewrite
```

A deletion ends with `*** Delete`:

```text
*** Begin Rewrite
*** Update File: src/obsolete.py
@@ remove the obsolete helper
40 | def legacy_helper():
... lines omitted ...
47 |     return cached_value
*** Delete
*** End Rewrite
```

`apply_rewrite` also supports file-level add, delete, update, and move operations, but its distinguishing feature is the numbered selection contract. It is the right tool when the model should prove which old span it is touching before it authors the replacement.

Full tool contract:

- [docutouch-server/tool_docs/apply_rewrite.md](docutouch-server/tool_docs/apply_rewrite.md)

## `apply_splice`

`apply_splice` treats existing text spans as transfer objects. A splice program selects a source span and then appends, inserts, replaces, moves, copies, or deletes that span.

A minimal copy-append program looks like this:

```text
*** Begin Splice
*** Copy From File: source.py
@@
12 | def build_context(...)
... source lines omitted ...
19 |     return "strict"
*** Append To File: target.py
*** End Splice
```

The selection is locked by absolute line numbers plus visible content. Omission markers are part of the syntax, not prose placeholders:

- source selections use `... source lines omitted ...`
- target selections use `... target lines omitted ...`

Use `apply_splice` when the target content already exists somewhere and the operation is about transfer relation rather than new text generation.

Full tool contract:

- [docutouch-server/tool_docs/apply_splice.md](docutouch-server/tool_docs/apply_splice.md)

## Reading And Search Workflow

DocuTouch's read path is intentionally file-boundary preserving. The default workflow for large context discovery is:

1. Use `list_directory` to establish the file map.
2. Use `search_text` or `structural_search` to narrow the relevant paths.
3. Use `read_file` on specific files and line ranges.
4. Use `apply_patch`, `apply_rewrite`, or `apply_splice` only after the evidence is stable.

`read_file` accepts a `line_range` in `start:stop` form. Positive endpoints are 1-indexed line numbers. Either endpoint may be omitted. Negative endpoints are relative to the end of the file.

Examples:

```text
:50     first 50 lines
50:     line 50 through EOF
-50:    last 50 lines
:-1     everything except the final line
```

Sampled inspection is separate from line ranges. Use `sample_step` and `sample_lines` when you need a sparse local view instead of one contiguous range.

`search_text` is the smart ripgrep entrypoint. `rg_args` accepts normal ripgrep flags, and the tool infers the result surface:

| `rg_args` intent | Output surface |
| --- | --- |
| default search | grouped results |
| `-A`, `-B`, `-C` | grouped context |
| `-c`, `--count`, `--count-matches` | counts |
| `-l`, `--files-with-matches`, `--files` | file list |
| `--json` | raw JSON |
| unsupported mixed layout | raw text |

`query_mode` defaults to `auto`: a regex parse failure falls back to literal search unless regex mode was explicitly requested.

## Structural Search

`structural_search` uses the `ast-grep` executable. It is available through MCP and supports these modes:

| Mode | Purpose |
| --- | --- |
| `find` | Run a pattern or rule query. |
| `expand` | Expand a previously registered result group. |
| `around` | Show local structural context around a result reference or `path:line`. |
| `explain_ast` | Explain the local AST shape for a result reference or `path:line`. |
| `rule_test` | Validate an ast-grep pattern or rule in a small scope. |

Result groups are registered inside the MCP connection. `ref="2"` points to group `[2]` from the most recent query; `ref="q1.2"` points to group `[2]` from query `q1`.

Rules must be passed as JSON objects. Edit-producing fields such as `fix`, `rewrite`, `replacement`, `apply`, `autofix`, and `transform` are rejected because DocuTouch uses structural search for evidence, not for ast-grep-driven mutation.

Full tool contract:

- [docutouch-server/tool_docs/structural_search.md](docutouch-server/tool_docs/structural_search.md)

## Installation

### GitHub Releases

The release workflow publishes platform binaries from tags named `v*`:

- `docutouch-x86_64-pc-windows-msvc.exe`
- `docutouch-x86_64-unknown-linux-gnu`
- `SHA256SUMS.txt`

Download them from:

- [GitHub Releases](https://github.com/MichengLiang/docutouch/releases)

On Linux or WSL, use the Linux binary for Linux paths. Launching the Windows `.exe` from WSL still gives the process Windows filesystem semantics, so a workspace like `/home/user/project` will not validate as a Windows path.

### Build From Source

Prerequisites:

- Rust toolchain and Cargo
- `rg` on `PATH` for `search_text`
- `ast-grep` on `PATH` for `structural_search`
- `pueue` and a running `pueued` service if you use `wait_pueue`

Build the workspace:

```bash
cargo build
```

Build the release binary:

```bash
cargo build --locked --release -p docutouch-server
```

The compiled binary is named `docutouch` because `docutouch-server/Cargo.toml` defines the binary target.

### npm Launcher

The `npm/` package is a thin launcher named `docutouch`.

```bash
npx docutouch help
npm install -g docutouch
```

The launcher supports Windows x64 and Linux x64. On first run it downloads the GitHub Release asset that matches the npm package version and caches the binary under the installed package's `vendor/` directory.

The npm dist-tag can lag behind GitHub Releases. If `npm view docutouch version` is older than the GitHub release you want, download the GitHub asset directly or build from source.

## MCP Configuration

For MCP hosts that use the common stdio configuration shape, point directly at a compiled binary when possible:

```json
{
  "mcpServers": {
    "docutouch": {
      "command": "/absolute/path/to/docutouch",
      "env": {
        "DOCUTOUCH_DEFAULT_WORKSPACE": "/absolute/path/to/project",
        "DOCUTOUCH_APPLY_PATCH_NUMBERED_EVIDENCE_MODE": "header_only"
      }
    }
  }
}
```

During local development, using `cargo run` is fine:

```json
{
  "mcpServers": {
    "docutouch": {
      "command": "cargo",
      "args": ["run", "-q", "-p", "docutouch-server"],
      "env": {
        "DOCUTOUCH_DEFAULT_WORKSPACE": "/absolute/path/to/project"
      }
    }
  }
}
```

Through npm:

```json
{
  "mcpServers": {
    "docutouch": {
      "command": "npx",
      "args": ["-y", "docutouch"],
      "env": {
        "DOCUTOUCH_DEFAULT_WORKSPACE": "/absolute/path/to/project"
      }
    }
  }
}
```

If your host calls `set_workspace` immediately after connecting, `DOCUTOUCH_DEFAULT_WORKSPACE` can be omitted.

## CLI Usage

Running `docutouch` with no subcommand starts the stdio MCP server.

```bash
docutouch
docutouch mcp
docutouch serve
```

The local CLI is available through top-level subcommands:

```bash
docutouch list [path] [--max-depth N] [--show-hidden] [--include-gitignored]
docutouch read <path> [--line-range START:END] [--show-line-numbers]
docutouch search <query> <path> [more_paths...] [--rg-args '...']
docutouch wait-pueue [TASK_ID ...] [--mode any|all] [--timeout-seconds N]
docutouch patch [patch-file] [--numbered-evidence-mode header_only|full]
docutouch rewrite [rewrite-file]
docutouch splice [splice-file]
```

From source, prefix the same commands with `cargo run -p docutouch-server --`:

```bash
cargo run -p docutouch-server -- list docutouch-server/src
cargo run -p docutouch-server -- read README.md --line-range 1:40 --show-line-numbers
cargo run -p docutouch-server -- search apply_patch docutouch-server/src --view full
cargo run -p docutouch-server -- wait-pueue 42 --mode any
cargo run -p docutouch-server -- read pueue-log:42
cargo run -p docutouch-server -- patch retry.patch
cargo run -p docutouch-server -- rewrite rewrite.txt
cargo run -p docutouch-server -- splice splice.txt
```

`patch`, `rewrite`, and `splice` read from stdin when no file argument is provided.

```bash
cat retry.patch | cargo run -p docutouch-server -- patch
cat rewrite.txt | cargo run -p docutouch-server -- rewrite
cat splice.txt | cargo run -p docutouch-server -- splice
```

For `.docutouch/failed-patches/*.patch` repair artifacts, the `patch` CLI restores the owning workspace anchor automatically before replaying the file.

## Environment Variables

| Variable | Applies to | Meaning |
| --- | --- | --- |
| `DOCUTOUCH_DEFAULT_WORKSPACE` | MCP server | Default base path for relative paths. |
| `DOCUTOUCH_APPLY_PATCH_NUMBERED_EVIDENCE_MODE` | MCP and CLI patch runtime | `header_only` or `full`; CLI `--numbered-evidence-mode` can override it for one patch invocation. |
| `DOCUTOUCH_PUEUE_BIN` | `wait_pueue` | Path or command name for the `pueue` executable. Defaults to `pueue`. |
| `DOCUTOUCH_PUEUE_RUNTIME_DIR` | `wait_pueue` | Overrides Pueue state/runtime directory resolution. |
| `DOCUTOUCH_PUEUE_TIMEOUT_SECONDS` | `wait_pueue` | Default wait timeout when an invocation does not pass `timeout_seconds`. |
| `PUEUE_CONFIG_PATH` | `wait_pueue` | Native Pueue config path used during runtime path discovery. |

## GitHub Release Flow

DocuTouch uses GitHub as the public distribution and automation surface.

The release workflow runs on tags matching `v*`:

1. Check out the repository.
2. Install stable Rust.
3. Build `docutouch-server` in release mode on Windows and Ubuntu runners.
4. Stage the platform-specific binary asset.
5. Upload artifacts.
6. Generate `SHA256SUMS.txt`.
7. Publish a GitHub Release with generated release notes.

The npm workflow runs after the `release` workflow completes successfully on a `v*` tag, or by manual dispatch with an explicit tag:

1. Check out the release tag from `workflow_run.head_branch`, or the manually supplied tag.
2. Install Node 24.
3. Verify `v${npm/package.json.version}` matches the release tag.
4. Run `npm publish --provenance --access public` from `npm/`.

That means a healthy public release has three matching versions: the Git tag, `docutouch-server/Cargo.toml`, and `npm/package.json`.

## Project Map

| Path | Role |
| --- | --- |
| [docutouch-core/](docutouch-core/) | Shared Rust library for filesystem surfaces, search, rewrite/splice runtime, patch presentation, and structural search. |
| [docutouch-server/](docutouch-server/) | Binary crate for the stdio MCP server and local CLI adapter. |
| [docutouch-server/tool_docs/](docutouch-server/tool_docs/) | Embedded tool descriptions used by the MCP server. |
| [codex-apply-patch/](codex-apply-patch/) | Vendored patch runtime fork with upstream lineage notes. |
| [npm/](npm/) | Node launcher package for GitHub Release binaries. |
| [docs/guide/](docs/guide/) | Reader-facing guides for quickstart, CLI, MCP, and tool surfaces. |
| [docs/source/](docs/source/) | More formal source documentation and knowledge records. |
| [example/](example/) | Small examples and MCP inspection scripts. |
| [script/](script/) | Local helper scripts and demos. |

## Documentation

Start here:

- [docs/guide/quickstart.md](docs/guide/quickstart.md)
- [docs/guide/tool-surfaces.md](docs/guide/tool-surfaces.md)
- [docs/guide/mcp-server.md](docs/guide/mcp-server.md)
- [docs/guide/cli.md](docs/guide/cli.md)
- [docs/guide/apply-patch-and-apply-splice.md](docs/guide/apply-patch-and-apply-splice.md)
- [docutouch-server/README.md](docutouch-server/README.md)
- [docutouch-core/README.md](docutouch-core/README.md)
- [codex-apply-patch/README.md](codex-apply-patch/README.md)

When a contract, diagnostic surface, CLI argument, or public tool boundary changes, update the matching embedded tool doc and reader-facing guide at the same time.

## Development

Run the main workspace tests:

```bash
cargo test -p docutouch-core -p docutouch-server
```

Run the server smoke tests only:

```bash
cargo test -p docutouch-server
```

Run the vendored patch engine tests:

```bash
cd codex-apply-patch
cargo test
```

Useful inspection commands:

```bash
cargo run -q -p docutouch-server -- help
cargo metadata --no-deps --format-version 1
npm view docutouch version dist-tags --json
```

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) before changing public behavior. The short version is:

- keep PRs focused
- state whether the change affects CLI, MCP, diagnostics, or tool contracts
- update docs when public surfaces change
- preserve the object boundary between patch, rewrite, splice, search, and read tools
- include tests for behavior changes and smoke coverage for CLI/MCP surfaces

## Status

The main MCP and CLI tool surfaces are usable. The project is still a fast-moving workbench: contracts are documented, release automation exists, and public packaging is in place, but the documentation and product boundary continue to be refined as the tool family grows.

## License

Apache-2.0. See [LICENSE](LICENSE).
