# DocuTouch

Language:

- [English](README.md)
- [简体中文](README.zh-CN.md)

DocuTouch is a set of structural file tools for coding-agent workflows.

The center of the project is two editing tools:

- `apply_patch`
- `apply_splice`

`apply_patch` keeps the familiar patch-shaped input model and strengthens the execution path for agent repair loops. `apply_splice` treats already-existing text spans as first-class objects for copy, move, delete, and replace operations.

Support tools remain available around that workflow:

- `list_directory`
- `read_file`
- `search_text`

## When To Reach For DocuTouch

DocuTouch is a good fit when you need to:

- express file changes as a structured patch instead of a prose editing request
- let correct, independent parts of a large patch land first while isolating the failing parts for repair
- turn a failed patch into a direct repair loop instead of regenerating the whole thing
- move, reuse, or reorganize already-existing code/text spans without re-authoring the full body

## `apply_patch`

`apply_patch` is the patch-shaped structural write tool. It still uses the familiar authored shape:

```text
*** Begin Patch
*** Update File: src/app.py
@@
-print("Hi")
+print("Hello")
*** End Patch
```

When ordinary context is not unique enough, `apply_patch` also supports one optional numbered anchor such as `@@ 120 | def handler():`.

The default numbered-evidence mode is `header_only`: numbered `@@` headers are interpreted, while body text that happens to look like `121 | value = 1` remains ordinary text by default. An advanced `full` mode exists for denser old-side numbered evidence when a human operator explicitly enables it.

In this repository, the runtime behavior goes beyond a direct upstream carry-over. The most important additions are:

- atomic commit inside connected file groups
- `PartialSuccess` across disjoint groups
- diagnostics designed for the next repair step
- separate warning blocks on successful runs
- a file-backed repair loop after patch failure

That means a large patch does not need to be retried from scratch when one independent part fails. Committed and failed parts are reported separately.

Warnings are rendered as explicit code-bearing blocks. For example:

```text
Success. Updated the following files:
M notes.md

warning[ADD_REPLACED_EXISTING_FILE]: Add File targeted an existing file and replaced its contents
  --> notes.md
  = help: prefer Update File when editing an existing file
```

Partial failure follows a repair-first shape. The headline starts with a stable error code:

```text
error[PARTIAL_UNIT_FAILURE]: patch partially applied
```

Then the output continues with the accounting needed for the next move:

- `committed changes:`
- `failed file groups:`
- `failed_group[n]`
- `attempted changes:`
- `help:`

The practical benefit is straightforward:

- retry less of what already succeeded
- resend less already-committed patch content
- spend fewer tokens on repeated context
- focus the next repair round on the failing group only

The CLI is also shaped around that repair loop. `patch` accepts both stdin and patch files. For `.docutouch/failed-patches/*.patch` repair artifacts, the CLI restores the owning workspace anchor so you can edit and replay directly.

See also:

- [codex-apply-patch/README.md](codex-apply-patch/README.md)
- [codex-apply-patch/UPSTREAM_LINEAGE.md](codex-apply-patch/UPSTREAM_LINEAGE.md)
- [codex-apply-patch/LOCAL_DIVERGENCES.md](codex-apply-patch/LOCAL_DIVERGENCES.md)

## `apply_splice`

`apply_splice` is a separate tool with a different object boundary.

- `apply_patch` operates on text differences and new text state
- `apply_splice` operates on transfer relations between already-existing text spans

Its authored surface declares:

- where an existing span comes from
- whether that span is `Copy`, `Move`, or `Delete Span`
- whether the destination side is `Append`, `Insert Before`, `Insert After`, or `Replace`

A minimal shape looks like this:

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

The important properties are:

- line-bearing selections
- absolute line numbers plus visible content as double-lock validation
- omission markers as first-class syntax
- `Delete Span` as a first-class action
- atomic connected mutation units

This is useful for model workflows because the tool can express reuse and relocation of existing text without restating the entire body as newly authored text.

## How To Choose Between The Two

Use `apply_patch` when the job is “change this text into another text state.”

Typical cases:

- rewrite a function body
- create a new file
- delete a file
- rename a file and change its contents

Use `apply_splice` when the job is “copy, move, delete, or replace a span that already exists.”

Typical cases:

- move a helper from one file to another
- copy an existing config block into a target file
- delete a contiguous existing span
- replace one existing block with another existing block

## Installation And Prerequisites

The primary public installation path is GitHub Releases plus source build.

Prerequisites:

- Rust toolchain
- Cargo
- `rg` (ripgrep) on PATH if you want to use `search_text`

Build the workspace:

```bash
cargo build
```

Release binaries are intended to ship as:

- `docutouch-x86_64-pc-windows-msvc.exe`
- `docutouch-x86_64-unknown-linux-gnu`
- `SHA256SUMS.txt`

The repository is also prepared for a scoped npm wrapper package:

```text
docutouch
```

The repo-side npm trusted-publishing workflow can be committed here, but npm still requires the package-side trusted publisher binding to be completed in npm package settings before fully automated publishing will succeed.

## Quick Start

Once a GitHub Release exists, you can also download the platform binary directly from the release assets instead of building from source.

Start the stdio MCP server:

```bash
cargo run -p docutouch-server -- serve
```

If you run the bare command instead:

```bash
cargo run -p docutouch-server
```

it now prints CLI usage. Use `serve` when you want the stdio MCP server.

Call the CLI directly:

```bash
cargo run -p docutouch-server -- list docutouch-server/src
cargo run -p docutouch-server -- read README.md --line-range 1:40
cargo run -p docutouch-server -- search apply_patch docutouch-server/src --view full
```

## MCP Config Example

Minimal stdio MCP server config:

```json
{
  "command": "cargo",
  "args": ["run", "-q", "-p", "docutouch-server"],
  "env": {
    "DOCUTOUCH_DEFAULT_WORKSPACE": "/absolute/path/to/project",
    "DOCUTOUCH_APPLY_PATCH_NUMBERED_EVIDENCE_MODE": "header_only"
  }
}
```

If your host will call `set_workspace` immediately after connecting, `DOCUTOUCH_DEFAULT_WORKSPACE` can be omitted.

## CLI Repair Loop

`patch` accepts both stdin and patch files. A typical repair loop is:

```bash
cargo run -p docutouch-server -- patch .docutouch/failed-patches/1712345678901-0.patch
```

If you need dense body-level numbered old-side evidence for one replay, you can enable it per invocation:

```bash
cargo run -p docutouch-server -- patch --numbered-evidence-mode full retry.patch
```

Or replay from stdin after editing the patch text:

```bash
cat retry.patch | cargo run -p docutouch-server -- patch
```

Once the scoped npm wrapper is published, the intended Node-side entry points are:

```bash
npx docutouch --help
npm install -g docutouch
```

The npm wrapper is designed as a thin launcher over GitHub Release binaries rather than a second implementation of the tool.

For `.docutouch/failed-patches/*.patch` repair artifacts, the CLI restores the owning workspace anchor automatically.

## Support Tools

- `list_directory`
  Builds an ASCII tree view of the workspace.

- `read_file`
  Reads one file at a time, with line ranges, line numbers, and sampled inspection.

- `search_text`
  Wraps ripgrep and returns grouped file-oriented search results.

## Documentation

- [docs/README.md](docs/README.md)
  Documentation entry.

- [docs/guide/README.md](docs/guide/README.md)
  Reader-facing guide entry.

- [docutouch-server/README.md](docutouch-server/README.md)
  `docutouch-server` package entry.

- [docutouch-core/README.md](docutouch-core/README.md)
  `docutouch-core` package entry.

- [codex-apply-patch/README.md](codex-apply-patch/README.md)
  `codex-apply-patch` package entry.

## Project Status

The core tool surfaces are usable today, while the public-facing docs and external presentation are still being tightened.

## Contributing

- [CONTRIBUTING.md](CONTRIBUTING.md)
  Contribution guide and documentation expectations.

## Tests

Run the core/server checks:

```bash
cargo test -p docutouch-core -p docutouch-server
```

Run the vendored patch engine tests:

```bash
cd codex-apply-patch
cargo test
```

## License

Apache-2.0.
