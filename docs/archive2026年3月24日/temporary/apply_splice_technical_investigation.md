# `apply_splice` Technical Investigation

## Status

- Pre-implementation investigation
- Based on a codebase review plus a QOC-style option evaluation
- Intended to narrow implementation risk before coding begins
- Product identity and narrow tool boundary are now fixed in
  `docs/apply_splice_spec.md`; this document should be read as architecture and
  reuse rationale, not as an invitation to renegotiate those boundaries

## 1. Executive recommendation

`apply_splice` should be implemented as a new DocuTouch-owned tool stack, not as
an extension of the vendored `codex-apply-patch` grammar/runtime.

The lowest-drift path is:

1. keep patch parsing/matching inside `codex-apply-patch`
2. extract a small shared mutation substrate for path identity and atomic commit
3. build `apply_splice` parser, selection resolver, runtime, and presentation on
   top of that shared substrate in DocuTouch-owned code

This keeps shared implementation where it matters, but avoids turning splice into
another product face of `apply_patch`.

## 2. Why this investigation exists

`apply_splice` is narrower than `apply_patch`, but it is not trivial.

It touches:

- grammar design
- numbered excerpt parsing
- source/target selection resolution
- cross-file atomic move/copy behavior
- diagnostics and repair accounting

Because of that, implementation should not begin from the current design spec
alone without a deeper reuse/refactor assessment.

## 3. QOC

Question:

- what implementation path gives `apply_splice` its intended narrow semantics
  while minimizing long-term semantic drift?

Criteria:

- upstream drift against the vendored patch fork
- semantic fit for existing-span structural operations
- reuse of atomicity/path-correctness work
- reuse of diagnostics and presentation style
- duplication risk
- implementation risk

### Option A. Extend `codex-apply-patch` directly

Assessment:

- easy access to existing private mutation/runtime helpers
- highest risk of upstream-drift and fork-boundary blur
- violates the intended product separation between patch and splice

Decision:

- reject

### Option B. Rebuild everything beside patch in DocuTouch code

Assessment:

- avoids touching the vendored fork
- duplicates the hardest correctness logic: path aliasing, commit grouping,
  rollback, and diagnostics taxonomy

Decision:

- acceptable only as a short-term spike, not as the preferred architecture

### Option C. Extract a shared mutation substrate, then build a DocuTouch-owned splice stack on top

Assessment:

- best balance between reuse and product separation
- preserves the ability to share path identity and atomic commit logic
- avoids forcing splice grammar into patch grammar

Decision:

- recommend

## 4. Direct reuse set

These existing pieces appear reusable with low semantic risk.

### 4.1 Path display helpers

- `docutouch-core/src/path_display.rs`

Reusable value:

- display path relativization
- compact scope rendering

### 4.2 Thin-adapter server/CLI patterns

- `docutouch-server/src/server.rs`
- `docutouch-server/src/cli.rs`

Reusable value:

- tool registration pattern
- workspace / CWD anchoring pattern
- MCP/CLI dual-surface structure

### 4.3 Test layering strategy

Reusable value:

- runtime tests in lower layers
- core presentation/outcome tests
- server smoke tests
- CLI parity tests

## 5. Reuse only after extraction/refactor

These pieces are valuable, but should not be consumed in their current private,
patch-owned shape.

### 5.1 Mutation substrate

Current home:

- `codex-apply-patch/src/lib.rs`

Candidate reusable concepts:

- path normalization
- path identity keys
- connected commit-unit grouping
- staged write / rollback machinery

Recommendation:

- extract into a new lower shared crate or clearly shared lower module, not into
  `docutouch-core` in patch-owned form

### 5.2 Generic diagnostic rendering helpers

Current home:

- `docutouch-core/src/patch_runtime.rs`
- `docutouch-core/src/patch_presentation.rs`

Candidate reusable concepts:

- location rendering
- target-anchor rendering
- compact repair-oriented output grammar

Recommendation:

- extract only the generic helpers
- keep splice-specific codes and wording separate; do not assume a tool-managed artifact layer

### 5.3 Numbered excerpt codec

Current home:

- `docutouch-core/src/fs_tools.rs`

Need:

- a stable parser/renderer for `N | text` excerpt lines used in source/target
  selection blocks

Recommendation:

- extract a tiny numbered-excerpt codec rather than letting splice and read-file
  formatting drift independently

## 6. Explicit non-reuse set

These existing pieces should not be reused directly because their semantics are
too patch-specific or directly conflict with the splice contract.

### 6.1 Patch grammar and hunk parser

Do not reuse directly:

- `codex-apply-patch/src/parser.rs`

Reason:

- grammar is hard-wired around Add/Delete/Update hunks and patch leniency logic

### 6.2 Diff/context replacement logic

Do not reuse directly:

- diff-based update matching and replacement logic in `codex-apply-patch/src/lib.rs`

Reason:

- splice is not a diff application problem
- splice is a source/target range transfer problem

### 6.3 Patch-specific sidecar schemas

Do not reuse directly:

- patch-specific sidecar names and payload kinds in patch presentation

Reason:

- current product direction is to avoid tool-managed artifacts entirely; splice should not inherit patch-specific sidecar assumptions

### 6.4 Sampled-read horizontal truncation semantics

Do not reuse directly:

- `max_chars` / `...[N chars omitted]` behavior from sampled `read_file`

Reason:

- `apply_splice` selections must forbid horizontal truncation

## 7. Main unresolved design questions

The design is narrow enough to justify deeper investigation, but not yet narrow
enough for safe full implementation. The main unresolved points are:

### 7.1 Grammar lock

Still needed:

- exact envelope spelling
- exact source/target header spellings
- exact omission token spellings inside source and target blocks

### 7.2 Same-file move ordering

Still needed:

- a hard rule for how source and target ranges are resolved when both are in the
  same file
- an overlap legality matrix

### 7.3 Target existence semantics

Still needed:

- whether append can create a missing destination file
- whether insert/replace require the target file to exist

### 7.4 Newline fidelity

Still needed:

- explicit policy for preserving line endings and EOF newline state

### 7.5 Diagnostics blame model

Still needed:

- how source selection mismatch, target selection mismatch, overlap errors, and
  write failures should each map to stable blame locations

## 8. Recommended implementation sequence

1. Do a short design lock for grammar, omission tokens, same-file move ordering,
   overlap legality, target existence, and newline policy.
2. Extract a shared mutation substrate from the patch-owned lower layer.
3. Build a DocuTouch-owned splice parser and source map.
4. Build a numbered-excerpt selection resolver with double-lock validation.
5. Build splice runtime on top of the shared mutation substrate.
6. Build splice-specific outcome mapping and presentation.
7. Expose it through MCP and CLI.
8. Add the same three-layer test strategy used elsewhere: runtime, core
   presentation/outcome, server/CLI parity.

## 9. Recommendation on whether to start implementation now

Recommendation:

- do not start full end-to-end implementation yet

Reason:

- there is already enough clarity to begin a focused implementation-prep phase
- there is not yet enough clarity to safely begin parser/runtime coding without
  risking drift or avoidable rework

What is safe to start now:

- a design-narrowing phase
- extraction planning for shared mutation primitives
- a small exploration of numbered-excerpt parsing strategy

What should wait until after that:

- full grammar parser implementation
- runtime move/copy engine
- diagnostics surface implementation

## 10. Suggested next-step checklist

- Record the architectural decision that `apply_splice` stays out of the vendored
  `codex-apply-patch` crate.
- Record the direct reuse set, the extract-before-reuse set, and the explicit
  non-reuse set.
- Add a module map proposal for `splice_parser`, `splice_selection`,
  `splice_runtime`, `splice_presentation`, plus the shared mutation substrate.
- Add a small design-lock section to the temporary plan for the still-unresolved
  grammar and runtime questions.
- Only after those are fixed, decide whether the project is ready for actual
  implementation.
