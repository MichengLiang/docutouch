# `apply_patch` Confirmed Gaps Audit

## Status

- Recorded on 2026-03-23.
- Historical gap audit captured before the 2026-03-23 repair wave landed.
- Scope: only confirmed gaps and mismatches.
- This file intentionally excludes speculative root-cause theories and unresolved design choices.
- See `Closeout Update` below for the post-repair status of the shared local contract.

## Purpose

This note exists to prevent already-confirmed `apply_patch` contract gaps from
disappearing into chat history.

It records:

- what is already confirmed to be inconsistent or undesirable
- where the evidence lives
- what category of fix is implied

It does not decide final implementation shape where product choices are still
open.

## Confirmed Gap 1: Empty `Add File` is rejected locally

### Confirmed behavior

In the local DocuTouch fork, an `Add File` block with zero `+` lines is rejected
as `OUTER_INVALID_ADD_LINE`.

### Local evidence

- `codex-apply-patch/src/parser.rs`
  - local parser explicitly errors when `parsed_lines == 1` after `*** Add File:`
  - current local implementation therefore requires at least one `+` line
- `docutouch-core/src/patch_runtime.rs`
  - malformed Add File lines are classified as `OUTER_INVALID_ADD_LINE`
- `docutouch-core/src/patch_presentation.rs`
  - renders this as `error[OUTER_INVALID_ADD_LINE]: Add File block is malformed`
- `docutouch-server/tests/stdio_smoke.rs`
  - current server contract asserts the malformed-Add diagnostic path

### Why this is a real gap

- creating an empty file is not inherently unsafe
- current behavior is stricter than the desired local product direction
- current tool docs already use grammar wording that can be read as allowing zero
  `+` lines

### Fix category implied

- code change required
- doc change required
- tests must be updated from failure expectation to success-plus-warning or other
  finalized local contract

## Confirmed Gap 2: Upstream main currently allows empty `Add File`

### Confirmed behavior

As of the current public `openai/codex` `main` branch inspected on 2026-03-23,
the upstream parser no longer errors on zero-line `Add File`.

### Upstream evidence

- `https://raw.githubusercontent.com/openai/codex/main/codex-rs/apply-patch/src/parser.rs`
  - current upstream `parse_one_hunk()` for `Add File` stops when the next line is
    not prefixed with `+`
  - unlike the local fork, it does not add a `parsed_lines == 1` rejection branch

### Why this matters

- the local empty-file rejection is not required for upstream compatibility
- local behavior is now stricter than current upstream behavior on this point

### Fix category implied

- local semantics can be relaxed without claiming an upstream break is required

## Confirmed Gap 3: Existing-file updates rewrite newline behavior

### Confirmed behavior

Current local update logic rewrites updated file content through an LF-based
string pipeline and forces a trailing newline on the final output.

### Local evidence

- `codex-apply-patch/src/lib.rs`
  - `load_file_state()` reads text via `std::fs::read_to_string`
  - `apply_update_file_to_content_with_diagnostics()` splits with `split('\n')`
  - updated contents are reassembled with `join("\n")`
  - final output forces a trailing newline if absent
- `codex-apply-patch/tests/suite/tool.rs`
  - current upstream-style test explicitly asserts that updating a file without a
    final newline appends one

### Reproduced local effect

A local repro against `docutouch patch` confirmed:

- input bytes: `61-0D-0A-62-0D-0A-63-0D-0A`
- output bytes after a one-line update: `61-0D-0A-78-0A-63-0D-0A`

This confirms:

- CRLF can become mixed with LF inside one file after update
- the tool is not merely preserving existing newline style

### Why this is a real gap

- this violates the desired local rule that existing files should not be
  reformatted as a side effect of edit application
- it creates diff noise and potential mixed-newline files

### Fix category implied

- code change required
- doc change required
- regression tests required for CRLF, mixed newline, and EOF-without-newline

## Confirmed Gap 3B: Existing-file updates also rewrite EOF-without-newline state

### Confirmed behavior

When an existing file is updated through the current local update pipeline, the
result is forced to end with a trailing newline.

### Local evidence

- `codex-apply-patch/src/lib.rs`
  - `apply_update_file_to_content_with_diagnostics()` pushes a final empty line
    when the rebuilt line vector does not already end with one
  - the returned contents are then joined with `"\n"`, guaranteeing a final
    newline byte sequence
- `codex-apply-patch/tests/suite/tool.rs`
  - current test coverage explicitly asserts that updating a file without a final
    newline appends one

### Why this is a real gap

- this is part of the same broader formatting-takeover problem as newline-style
  rewriting
- the current local desired direction is that existing files should not be
  reformatted as a side effect of patch application
- EOF newline state therefore needs to be tracked as an explicit contract, not
  left as an accidental by-product of the line-join algorithm

### Fix category implied

- code change required
- doc change required
- regression tests required for EOF-without-newline preservation after the final
  local contract is locked

## Confirmed Gap 4: Newly created files are LF-authored by construction

### Confirmed behavior

Current local `Add File` construction appends `\n` after each `+` line, so newly
created files are emitted in LF form.

### Local evidence

- `codex-apply-patch/src/parser.rs`
  - `Add File` parsing pushes `'\n'` after each added line

### Why this matters

- this behavior is compatible with the desired local rule for newly created files
- therefore the newline problem is specifically about edits to existing files,
  not about file creation semantics

### Fix category implied

- preserve this behavior unless a broader product decision changes it

## Confirmed Gap 5: Multiple `@@` headers are documented but not supported by the parser

### Confirmed behavior

The local parser only supports at most one explicit `@@` header line at the start
of an update chunk. A second consecutive `@@ ...` line is treated as an invalid
update-hunk line.

### Local evidence

- `codex-apply-patch/src/parser.rs`
  - `parse_update_file_chunk()` consumes one optional header
  - after that, each line must begin with space, `+`, `-`, or `*** End of File`
  - a second `@@ ...` line is therefore invalid
- direct local repro against `docutouch patch`
  - a patch using consecutive `@@ class B`, `@@ def run(self):`, `@@ def inner():`
    fails with `error[OUTER_INVALID_LINE]`

### Why this is a real gap

- current active tool docs teach multi-`@@` header authoring
- runtime rejects the taught form

### Fix category implied

- either remove multi-`@@` guidance from active docs
- or implement actual parser support
- current state is an active prompt/runtime contract mismatch

## Confirmed Gap 6: The currently injected DocuTouch tool doc teaches unsupported multi-`@@` syntax

### Confirmed behavior

The active server-injected tool description is loaded from
`docutouch-server/tool_docs/apply_patch.md`, and that file currently teaches
multiple `@@` headers, including an indented example form.

### Local evidence

- `docutouch-server/src/tool_service.rs`
  - includes `../tool_docs/apply_patch.md` as the tool description
- `docutouch-server/tool_docs/apply_patch.md`
  - says multiple `@@` headers may be chained
  - includes the example `@@ \t def method():`

### Why this is a real gap

- the injected prompt surface teaches a form the parser does not accept
- this is not merely historical documentation drift; it is active host guidance

### Fix category implied

- prompt-doc correction required immediately unless parser support is added first

## Confirmed Gap 7: Upstream main still documents multiple `@@` headers

### Confirmed behavior

As of the current public `openai/codex` `main` branch inspected on 2026-03-23,
the upstream tool instructions still say multiple `@@` statements can be used,
while the upstream parser still only consumes one explicit header.

### Upstream evidence

- `https://raw.githubusercontent.com/openai/codex/main/codex-rs/apply-patch/apply_patch_tool_instructions.md`
  - still teaches that multiple `@@` statements may be used
- `https://raw.githubusercontent.com/openai/codex/main/codex-rs/apply-patch/src/parser.rs`
  - still only consumes one explicit `@@` header before body lines

### Why this matters

- this is not a DocuTouch-only hallucination or local doc-edit accident
- upstream itself currently appears to have the same prompt/parser mismatch on
  this point

### Fix category implied

- local project should still choose a healthy local contract instead of waiting
  for upstream alignment

## Confirmed Gap 8: Standalone `apply_patch` CLI contract still lags DocuTouch transport behavior

### Confirmed behavior

The vendored standalone `apply_patch` CLI tests still assert older outward
behavior on several paths where the DocuTouch MCP / `docutouch patch` surface has
already moved to healthier diagnostics or warnings.

### Local evidence

- `codex-apply-patch/tests/suite/tool.rs`
  - empty patch still expects `No files were modified.`
  - overwrite-via-Add success still expects no warning text
  - overwrite-via-Move success still expects no warning text
- `docutouch-server/tests/stdio_smoke.rs`
  - MCP surface expects overwrite warnings to be rendered

### Why this is a real gap

- transport-level behavior is no longer cleanly unified
- if the standalone binary remains supported, this drift is technical debt rather
  than harmless variance

### Fix category implied

- either align transports
- or explicitly document the standalone path as intentionally different

## Confirmed Gap 9: Newline regression coverage is missing in the direction the local product now needs

### Confirmed behavior

Current automated coverage does not materially protect the desired local
non-formatting rule for existing files.

### Local evidence

- no substantial CRLF-preservation regression coverage found across:
  - `codex-apply-patch`
  - `docutouch-core`
  - `docutouch-server`
- current durable coverage instead includes at least one test that asserts
  trailing-newline insertion on update

### Why this is a real gap

- known newline-formatting side effects remain easy to regress
- the test suite is not yet enforcing the intended healthier local contract

### Fix category implied

- add regression tests for:
  - CRLF-preserving updates
  - mixed-newline preservation policy after final decision
  - EOF-without-newline preservation policy after final decision
  - empty `Add File` once the new contract is chosen
  - multi-`@@` header behavior after the contract is chosen

## Summary

The following points are now confirmed and should be treated as real tracked
gaps, not loose impressions:

1. local empty-file creation via `Add File` is unnecessarily rejected
2. current local update logic rewrites newline behavior in existing files
3. current local update logic also rewrites EOF-without-newline state
4. newly created files already naturally emit LF
5. multiple `@@` headers are currently documented but unsupported
6. the active injected tool doc teaches unsupported syntax
7. upstream main currently appears to have the same multi-`@@` prompt/parser mismatch
8. local behavior on empty `Add File` is now stricter than upstream main
9. standalone `apply_patch` CLI behavior lags the healthier DocuTouch surface
10. regression coverage does not yet protect the newline contract that local DX now needs

## Owner-Confirmed Direction Already Established

The following directional decisions are no longer merely analytical suggestions.
They have been explicitly confirmed in discussion and should be treated as
implementation-driving constraints for subsequent repair planning.

1. Empty `Add File` rejection is to be removed locally.
2. Existing-file newline and EOF newline takeover are to be repaired in both code
   and docs.
3. Regression protection for newline behavior is to be added rather than deferred.
4. Standalone `apply_patch` transport drift is technical debt to be repaired,
   rather than tolerated in the name of upstream alignment.
5. The current active multiple-`@@` prompt/runtime mismatch must not remain
   undocumented or silently tolerated.

These points are recorded here only as confirmed remediation direction for the
defects already listed above. Detailed design rationale and future-interface
discussion belong in separate documents.

## Closeout Update

As of the 2026-03-23 contract-repair execution wave, the current local
implementation direction for the primary DocuTouch `apply_patch` surface has
been executed rather than merely planned.

Closed local repair outcomes:

1. empty `Add File` rejection has been removed locally
2. existing-file update no longer silently rewrites newline style
3. existing-file update no longer silently takes over EOF-without-newline state
4. active prompt/runtime drift around stacked multiple `@@` teaching has been
   narrowed on the prompt-facing path
5. server-facing regression coverage now protects the repaired contract

Remaining distinction to keep visible:

- newly created files remain LF-authored by construction unless a later product
  decision changes that rule
- the vendored raw standalone `apply_patch` executable remains a separate
  outward surface whose UX should stay aligned, but that transport concern is no
  longer the same question as whether the local core contract repair set landed
