# `apply_splice` Architecture / Diagnostics / QA Draft

## Status

- Scope: closure draft for architecture boundary, shared mutation substrate, diagnostics contract, source-of-truth policy, and recursive QA model
- Intentionally out of scope: canonical grammar lock and full authored-surface wording
- The stable product boundary already lives in `docs/apply_splice_spec.md`; this
  draft exists to close architecture, diagnostics, and QA obligations on top of it

## 1. Architecture Boundary

### 1.1 Architectural intent

`apply_splice` should ship as a DocuTouch-owned tool with a public identity separate from `apply_patch`, while reusing the lowest shared correctness substrate that already exists in the patch stack. This is the same direction recommended by the technical investigation and closure summary: keep patch parsing/matching inside the vendored fork, extract a small shared mutation substrate, and build splice parser/selection/runtime/presentation on top of it (`docs/temporary/apply_splice_technical_investigation.md:11-19`, `docs/temporary/apply_splice_closure/stage_summary.md:11-27`).

### 1.2 What may be shared

The following capabilities are good shared candidates because they are correctness substrate, not product identity:

| Area | Share? | Why |
| --- | --- | --- |
| Path normalization and path identity keys | Yes | Already required to collapse alias paths and prevent accidental split commit units (`codex-apply-patch/src/lib.rs:667-675`, `codex-apply-patch/src/lib.rs:1320-1345`) |
| Connected-unit grouping | Yes | This is the atomicity backbone and should not be reimplemented independently (`codex-apply-patch/src/lib.rs:678-729`, `codex-apply-patch/DOCUTOUCH_ENHANCEMENTS.md:7-18`) |
| Staged write / rollback / commit planning | Yes | Multi-path move/copy safety is the same class of problem for patch and splice (`codex-apply-patch/DOCUTOUCH_ENHANCEMENTS.md:43-46`) |
| Affected-path summarization (`A/M/D`) | Yes, as a generic renderer/helper | The family-level success contract is already stable and should stay aligned (`docutouch-server/tool_docs/apply_patch.md:52-58`) |
| Path display / relativization | Yes | Pure presentation helper with no splice-specific semantics (`docutouch-core/src/path_display.rs:3-37`) |
| Generic diagnostic rendering helpers | Yes, after extraction | Location rendering, target-anchor rendering, and inline failure-layout helpers are cross-tool concerns (`docutouch-core/src/patch_presentation.rs:255-493`) |
| Numbered excerpt codec | Yes, but only as a tiny codec | `apply_splice` should not drift away from DocuTouch line-number rendering conventions, but the codec must stay independent from sampled-view truncation semantics (`docs/temporary/apply_splice_technical_investigation.md:165-179`, `docs/read_file_sampled_view_spec.md:169-201`) |

### 1.3 What must remain splice-owned

The following must stay outside the vendored patch crate and outside any generic shared layer:

| Area | Ownership rule | Why |
| --- | --- | --- |
| Public tool name, tool docs, and prompt-facing contract | Splice-owned | Product identity must stay separate even if some code is shared (`docs/apply_splice_spec.md:317-353`) |
| Envelope grammar and parser | Splice-owned | Patch grammar is hunk/diff-centric and is explicitly non-reusable for splice (`docs/temporary/apply_splice_technical_investigation.md:186-205`) |
| Source/target selection resolver | Splice-owned | Double-lock numbered excerpt validation is specific to splice semantics (`docs/apply_splice_spec.md:161-242`) |
| Same-file snapshot policy and overlap legality | Splice-owned | These are semantic decisions, not filesystem substrate (`docs/apply_splice_spec.md:387-423`) |
| Source-byte preservation and newline fidelity policy | Splice-owned | This is part of splice's transfer semantics for existing spans (`docs/apply_splice_spec.md:428-439`) |
| Splice error codes, summaries, and diagnostic vocabulary | Splice-owned | Wording and any splice-owned diagnostic kinds should match DocuTouch style without pretending patch and splice are the same tool (`docs/temporary/apply_splice_technical_investigation.md:147-163`) |
| Transfer/removal action semantics | Splice-owned | The tool's semantic core now consists of the eight transfer actions plus the source-only `Delete Span` primitive (`docs/apply_splice_spec.md:93-129`, `docs/temporary/apply_splice_closure/stage_summary.md:11-26`) |

### 1.4 Boundary rule

Requirement: shared code may solve only these cross-tool concerns:

- path identity and alias collapse
- connected mutation unit planning
- staged commit / rollback
- generic display helpers
- generic diagnostic rendering helpers

Requirement: if a helper needs to know whether a selection is `source` vs `target`, whether overlap is legal, or whether newline bytes must be preserved verbatim, it is already too high-level to live in the shared substrate.

## 2. Suggested Module Map

The recommended map is intentionally narrow. It preserves a small shared core and keeps all splice semantics above that line.

### 2.1 Shared lower layer

1. `docutouch-mutation` (new crate or clearly isolated lower module)
   - `path_identity.rs`
   - `commit_units.rs`
   - `staged_commit.rs`
   - `affected_paths.rs`

2. `docutouch-core/src/numbered_excerpt.rs`
   - parse/render `N | text`
   - vertical omission marker handling
   - explicit rejection hooks for horizontal truncation in strict consumers

3. `docutouch-core/src/diagnostic_render.rs`
   - display-path adaptation
   - primary location rendering
   - optional target-anchor rendering
   - repair-first failure-layout helpers
   - generic committed / attempted `A/M/D` line rendering

### 2.2 Splice-owned layer

1. `docutouch-core/src/splice_parser.rs`
   - envelope parsing
   - source map capture for action headers and selection blocks

2. `docutouch-core/src/splice_selection.rs`
   - numbered excerpt decoding
   - contiguous-span reconstruction
   - double-lock validation
   - overlap detection inputs

3. `docutouch-core/src/splice_runtime.rs`
   - source/target resolution against one original snapshot
   - transfer and delete execution planning
   - translation into shared mutation units
   - splice-specific warnings and failure payloads

4. `docutouch-core/src/splice_presentation.rs`
   - splice success/failure wording
   - splice diagnostic code families and summary wording
   - final user-visible repair surface

5. `docutouch-server`
   - tool registration
   - MCP/CLI parity
   - `tool_docs/apply_splice.md`

This shape fits the current workspace split where `docutouch-core` owns shared tool semantics and presentation, `docutouch-server` is a thin adapter, and the vendored crate stays focused on upstream patch lineage plus extracted lower-level correctness work (`README.md:7-31`, `docutouch-core/src/lib.rs:1-15`).

## 3. Diagnostics Contract

### 3.1 Obligations

`apply_splice` diagnostics should inherit the current DocuTouch philosophy:

- stable codes
- compact repair-oriented rendering
- truthful blame hierarchy
- preserved repair accounting for partial success
- self-contained inline failures with no tool-managed sidecar requirement

This is already the explicit target for patch diagnostics and should remain the family baseline, not a patch-only quirk (`docs/apply_patch_diagnostics_spec.md:59-90`, `docs/apply_splice_spec.md:298-316`).

### 3.2 Minimal code family

The minimum viable splice diagnostic family should be small but complete enough to close the main semantic risks:

| Code family | Minimum meaning | Primary blame location |
| --- | --- | --- |
| `SPLICE_SOURCE_SELECTION_INVALID` | source numbered excerpt does not resolve truthfully | source selection block |
| `SPLICE_TARGET_SELECTION_INVALID` | target numbered excerpt does not resolve truthfully | target selection block |
| `SPLICE_SELECTION_TRUNCATED` | horizontal truncation or invalid omission form detected inside a selection | offending selection line |
| `SPLICE_OVERLAP_ILLEGAL` | same-file overlap violates v1 legality rule | action header |
| `SPLICE_TARGET_STATE_INVALID` | required target file/range does not exist for the requested action | target selection block or action header |
| `SPLICE_WRITE_ERROR` | commit-stage filesystem failure after truthful planning | target path or destination anchor when no stronger authored span exists |
| `SPLICE_PARTIAL_UNIT_FAILURE` | some connected units committed while others failed | first failing unit plus committed/failed-group summary |

This is intentionally a family, not a huge enum. More fine-grained subcodes can come later if repair quality proves they are necessary.

### 3.3 Required rendering behavior

Requirement statements:

1. Every failure must start with `error[CODE]: ...`.
2. The primary blame location must follow the splice-specific hierarchy already locked in the spec:
   - source selection location for source mismatch
   - target selection location for target mismatch
   - action header for semantic errors such as overlap or ordering
   - target path / destination anchor for write-stage failures when no stronger authored span exists
3. One optional secondary target anchor is allowed only when the causal chain is strong, following the same honesty rule used by patch diagnostics (`docs/apply_patch_diagnostics_spec.md:119-208`, `docutouch-core/src/patch_runtime.rs:314-407`, `docutouch-core/src/patch_presentation.rs:461-493`).
4. Partial failure must preserve:
   - committed `A/M/D`
   - failed units/groups
   - attempted `A/M/D` for each failed unit
   - repair-oriented `help:` lines
5. Sidecars may exist for overflow, but splice must use its own schema identity such as `docutouch.apply_splice.failed_groups.v1`, not the patch kind name (`docutouch-core/src/patch_presentation.rs:179-252`).

### 3.4 Success-path compatibility

Success output should stay aligned with the existing tool family:

- compact `A/M/D` summary remains the default visible success shape
- move-shaped success should normalize to destination-side `M`, matching the current implemented patch baseline (`codex-apply-patch/src/lib.rs:940-955`, `docs/temporary/apply_splice_closure/stage_summary.md:21-27`)
- finer distinctions belong in warnings and failure diagnostics, not in a custom verbose success dialect (`docutouch-server/tool_docs/apply_patch.md:52-58`)

## 4. Source-of-Truth / Closure Policy

### 4.1 Pre-implementation policy

Before `apply_splice` code exists, the closure source of truth is the closed decision set captured in:

- `docs/apply_splice_spec.md`
- `docs/temporary/apply_splice_technical_investigation.md`
- `docs/temporary/apply_splice_closure/stage_summary.md`

Chat logs and draft examples are supporting evidence only. They do not override a closed decision.

### 4.2 Post-implementation policy

Once splice code ships, precedence should be:

1. Executable behavior in code plus passing tests
2. Stable design/spec documents for intended contract
3. Tool docs, README examples, and prose examples
4. Temporary notes and chat excerpts

This mirrors the closure summary decision that when docs/examples drift from the actual runtime, code plus tests are the current runtime source of truth (`docs/temporary/apply_splice_closure/stage_summary.md:25-27`). It also matches the maintainer rule that behavior changes must update docs, rather than letting chat residue stand in for project truth (`docs/maintainer_guide.md:43-66`).

### 4.3 Drift handling requirement

Requirement: if tests and docs disagree, the owner must do one of two things in the same workstream:

- update the stale docs/examples to the implemented behavior, or
- change the implementation/tests to the intended behavior and record the decision

Requirement: no example snippet in README/tool docs may overrule passing runtime tests for observable behavior.

Requirement: if the runtime intentionally diverges from upstream patch lineage in a new shared substrate area, the divergence must be recorded the same way `codex-apply-patch` already records local enhancements (`codex-apply-patch/UPSTREAM_BASELINE.md:8-17`, `codex-apply-patch/DOCUTOUCH_ENHANCEMENTS.md:1-49`).

## 5. Recursive QA / Test Model

### 5.1 Why the test model must be recursive

The closure review explicitly called out the absence of a recursive test model (`docs/temporary/apply_splice_closure/review_log.md:18-27`). The fix is not "add more integration tests." The fix is to decompose each requirement down the stack and verify it at the cheapest truthful layer first, then reassert it one layer outward.

### 5.2 Decomposition rule

For every top-level requirement, derive tests in this order:

1. shared substrate invariant
2. splice semantic invariant
3. user-visible presentation invariant
4. transport/parity invariant

Example:

- Requirement: same-file move must resolve source and target against one original snapshot.
- Shared substrate test: commit-unit planner preserves one before-state and one after-state.
- Splice semantic test: resolver rejects overlap and does not interpret target against intermediate state.
- Presentation test: emitted error points to the action header and carries the stable overlap code.
- Server/CLI parity test: MCP and CLI surfaces expose the same failure wording and fields.

### 5.3 Layered test map

| Layer | Primary responsibility | Existing analog |
| --- | --- | --- |
| `docutouch-mutation` or equivalent shared layer | path identity, alias collapse, connected units, staged rollback, affected-path normalization | `codex-apply-patch` commit-unit tests (`codex-apply-patch/src/lib.rs:2289-2325`) |
| `splice_selection` | numbered excerpt parsing, double-lock validation, truncation rejection, same-file overlap inputs | `read_file` numbered-line and omission tests (`docutouch-core/tests/fs_tools.rs:53-226`) |
| `splice_runtime` | transfer and delete semantics, same-file snapshot policy, newline fidelity, target existence rules, partial-success behavior | current patch runtime/outcome tests (`docutouch-core/src/patch_runtime.rs:247-407`) |
| `splice_presentation` | headline, blame location, target anchor, committed/attempted `A/M/D`, repair-first failure layout | `docutouch-core/src/patch_presentation.rs:255-520` |
| `docutouch-server` MCP/CLI tests | transport contract, text parity, host-audit-neutral behavior, workspace handling | `docutouch-server/tests/cli_smoke.rs:141-220`, `docutouch-server/tests/stdio_smoke.rs:417-547`, `docutouch-server/tests/stdio_smoke.rs:1088-1398` |

This matches the existing three-layer maintenance rule, but adds one extra inner layer for the new numbered-excerpt and shared-mutation substrate work that patch did not have to expose separately (`docs/maintainer_guide.md:151-180`).

### 5.4 Minimum scenario matrix

The first implementation wave should not be considered closed unless the test plan covers at least these scenario families:

- all eight transfer combinations plus `Delete Span`
- cross-file vs same-file execution
- same-file legal vs illegal overlap
- source drift vs target drift
- target missing vs target present
- append-to-missing-file vs insert/replace-on-missing-file
- newline fidelity cases, including EOF-without-newline
- alias-path grouping and rollback behavior
- single connected unit failure vs multi-unit partial success
- diagnostics with and without target-anchor evidence
- MCP/CLI parity for both success and failure

### 5.5 Review rule

No new splice semantic rule is complete until it has:

- one positive test
- one nearest-boundary negative test
- one diagnostics assertion
- one outer-surface parity assertion if the user-visible message changes

That is the recursive QA model in operational form.

## Implementation Entry Criteria

- [ ] Shared substrate extraction boundary is named and documented: path identity, connected units, staged commit, affected-path summary, no splice semantics above that line.
- [ ] A numbered-excerpt codec contract exists and explicitly separates vertical omission from forbidden horizontal truncation.
- [ ] Splice-owned modules are named and scoped: parser, selection, runtime, presentation, server wiring.
- [ ] The minimal splice diagnostics code family is locked.
- [ ] Blame hierarchy is locked for source mismatch, target mismatch, overlap/semantic error, and write-stage failure.
- [ ] Success-path reporting is locked to family-compatible `A/M/D`, including destination-side `M` normalization for move-shaped success.
- [ ] Source-of-truth precedence is written down for pre-implementation and post-implementation drift handling.
- [ ] The recursive test decomposition is accepted, including layer ownership and minimum scenario matrix.
- [ ] The first implementation batch is able to add tests at every required layer, not only end-to-end.
- [ ] Grammar work is explicitly handed off to the separate canonical-grammar artifact rather than being re-opened inside architecture work.
