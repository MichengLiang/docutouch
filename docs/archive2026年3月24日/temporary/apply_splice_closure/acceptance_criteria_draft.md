# `apply_splice` Acceptance and QA Criteria Draft

## Status

- Purpose: implementation-driving acceptance contract for `apply_splice`
- Audience: implementer, reviewer, QA owner, and release gatekeeper
- Scope: release-entry criteria for grammar, semantics, runtime safety, diagnostics, transport parity, and documentation sync
- In this document, `v1` means the first accepted implementation closure of the
  stable contract; it does not reopen tool identity, action basis, or the narrow
  existing-span structural-operation boundary

## 1. Review Model

This document turns the current `apply_splice` closure work into evidence-backed release gates.

Review rule:

1. Every criterion in this document is a pass/fail gate.
2. A criterion passes only if every listed pass check succeeds and every listed evidence item exists in the same workstream.
3. Missing automated evidence counts as failure unless the criterion explicitly says manual evidence is acceptable.
4. A green end-to-end demo does not waive missing lower-layer evidence.
5. If code, tests, and docs disagree, the criterion fails until the disagreement is resolved in the same workstream.

This draft assumes stable product-boundary decisions already live in `docs/apply_splice_spec.md`.
The remaining temporary materials below are authoritative only for
implementation-facing details until they are promoted:

- `docs/temporary/apply_splice_technical_investigation.md`
- `docs/temporary/apply_splice_closure/stage_summary.md`
- `docs/temporary/apply_splice_closure/formal_semantics_draft.md`
- `docs/temporary/apply_splice_closure/architecture_diagnostics_test_draft.md`

Baseline runtime and UX evidence that `apply_splice` must align with where applicable:

- `codex-apply-patch/src/lib.rs:475-584` for commit-unit status model
- `codex-apply-patch/src/lib.rs:667-729` for connected-unit grouping by path identity
- `codex-apply-patch/src/lib.rs:940-955` for destination-side `M` normalization of move-shaped success
- `codex-apply-patch/src/lib.rs:1216-1298` for staged commit and rollback discipline
- `docutouch-core/src/fs_tools.rs:497-579` for numbered line rendering, vertical omission rendering, and line-ending splitting baseline
- `docutouch-server/tool_docs/apply_patch.md:52-88` for family-level success/failure shape expectations
- `docs/apply_patch_diagnostics_spec.md:42-90` for diagnostics philosophy and partial-failure repair accounting
- `docutouch-server/tests/cli_smoke.rs:195-220` and `docutouch-server/tests/stdio_smoke.rs:1088-1398` for CLI/MCP parity and failure-shape baselines

## 2. Required Evidence Classes

Unless a criterion says otherwise, required evidence must come from the following layers.

- `E1. Spec evidence`
  - Stable or temporary design/spec text updated to match the implemented behavior.
- `E2. Semantic unit evidence`
  - Lower-layer tests for parser, selection resolver, runtime planner, or shared mutation substrate.
- `E3. Presentation evidence`
  - Tests asserting rendered success/failure text, blame location, warnings, and host-audit-neutral transport behavior.
- `E4. Transport evidence`
  - MCP and CLI tests proving the same observable contract.
- `E5. Regression evidence`
  - A durable test added to the permanent suite so the behavior cannot silently drift.

Recommended test placement if the current module map is adopted:

- `docutouch-core/tests/splice_parser.rs`
- `docutouch-core/tests/splice_selection.rs`
- `docutouch-core/tests/splice_runtime.rs`
- `docutouch-core/tests/splice_presentation.rs`
- shared mutation tests beside the extracted substrate crate/module
- `docutouch-server/tests/cli_smoke.rs`
- `docutouch-server/tests/stdio_smoke.rs`

Equivalent placement is acceptable only if layer ownership remains equally explicit.

## 3. Top-Level Acceptance Goals

### AG-01 Narrow Tool Identity

`apply_splice` must remain a narrow structural-operation tool over existing spans: transfer actions preserve selected source bytes, delete actions remove selected existing spans, and inline authored text remains forbidden.

### AG-02 Deterministic Selection Semantics

Source and target selections must resolve by absolute line numbers plus visible-content matching, with contiguous denotation and no horizontal truncation.

### AG-03 Deterministic Same-File Behavior

Same-file programs must resolve against one original snapshot and must reject overlap or intermediate-state-dependent interpretation in v1.

### AG-04 Byte Fidelity

Transferred bytes must be the exact selected source bytes, including separator bytes and EOF-without-newline state.

### AG-05 Atomic and Accounted Mutation

Connected file sets must commit atomically, disjoint units may partially succeed, and the result must remain clearly accounted for in inline status/output.

### AG-06 Honest Diagnostics

Failures must have stable codes, truthful blame, repair-oriented wording, and no fabricated precision.

### AG-07 MCP/CLI Contract Parity

MCP and CLI must expose the same semantics, same visible text contract, and the same repair-first contract for the same splice program.

### AG-08 Documentation Truthfulness

Stable docs, tool docs, tests, and examples must describe the same contract that the runtime implements.

## 4. Decomposed Acceptance Criteria

## 4.1 Grammar and Authored Surface

### GRAM-01 Canonical envelope and action headers are exact

- Requirement
  - The only accepted v1 envelope is `*** Begin Splice` ... `*** End Splice`.
  - The only accepted source/action headers are `*** Copy From File: `, `*** Move From File: `, and `*** Delete Span From File: `.
  - The only accepted target headers are `*** Append To File: `, `*** Insert Before In File: `, `*** Insert After In File: `, and `*** Replace In File: `.
- Pass checks
  - Exact spellings parse successfully.
  - Any spelling drift, synonym, capitalization change, or extra token fails parse.
  - Parse failure occurs before any file mutation attempt.
- Required evidence
  - `E1`: grammar text in stable or closure-owned spec.
  - `E2`: parser tests for every accepted header and at least one rejection per near-miss spelling.
  - `E5`: regression tests for rejected variants.
- Negative-test expectations
  - Reject `*** BeginSplice`, `*** Copy File:`, `*** Insert Before File:`, trailing comments, and duplicate header lines.
  - Failure must identify the authored location of the malformed header.
- Regression requirement
  - Any future header addition or spelling change must update grammar spec, parser tests, and tool docs in the same change.

### GRAM-02 Action shape is structurally closed

- Requirement
  - Every action contains exactly one action header and exactly one source selection block.
  - `Delete Span` actions contain no target clause or target selection block.
  - `Append` actions contain no target selection block.
  - `Insert Before`, `Insert After`, and `Replace` contain exactly one target selection block.
- Pass checks
  - Valid shapes parse and plan.
  - Extra target selection or target clause on delete fails.
  - Missing target selection on anchored actions fails.
  - Extra target selection on append fails.
  - Duplicate source or target blocks fail.
- Required evidence
  - `E1`: grammar/well-formedness section.
  - `E2`: parser or AST validation tests covering each valid and invalid action shape.
  - `E5`: permanent negative tests for over-specified and under-specified actions.
- Negative-test expectations
  - Reject two source blocks in one action.
  - Reject append followed by `@@` target selection.
  - Reject anchored target header with no following selection.
- Regression requirement
  - The AST or validated action model must make illegal shapes unrepresentable after validation.

### GRAM-03 Selection-line syntax is strict and whitespace-significant

- Requirement
  - Selection lines use exact `PositiveInteger + " | " + VisibleContent` syntax.
  - Empty visible content is valid.
  - Leading and trailing spaces in visible content are semantically significant.
- Pass checks
  - `9 | ` parses as an empty visible line.
  - `9|x`, `9  | x`, `09 | x`, and negative or zero line numbers fail.
  - Tests prove that leading and trailing spaces survive parse and matching.
- Required evidence
  - `E1`: syntax definition in grammar/spec.
  - `E2`: parser tests for empty lines, leading-space lines, trailing-space lines, and invalid delimiters.
  - `E5`: raw fixture-based tests showing exact visible-content preservation.
- Negative-test expectations
  - Reject malformed delimiters and non-positive numbering.
  - Reject duplicate line numbers within one selection body.
- Regression requirement
  - Future formatting helpers must not auto-trim or normalize visible content in selections.

### GRAM-04 Omission tokens are explicit, side-specific, and structurally constrained

- Requirement
  - Source selections use only `... source lines omitted ...`.
  - Target selections use only `... target lines omitted ...`.
  - Omission tokens may appear only between numbered lines and only when the numeric gap is greater than 1.
- Pass checks
  - Valid omission compresses a contiguous range.
  - Bare `...` is rejected.
  - Wrong-side omission token is rejected.
  - Leading, trailing, adjacent, and empty-gap omission placements are rejected.
- Required evidence
  - `E1`: omission-token contract in grammar/spec.
  - `E2`: selection parser/resolver tests for valid and invalid omission placement.
  - `E5`: regression tests for bare `...` and wrong-token-kind rejection.
- Negative-test expectations
  - Reject `...` copied from sampled `read_file` output.
  - Reject `... source lines omitted ...` inside target selections.
  - Reject omission between adjacent numbered lines.
- Regression requirement
  - Any future compatibility relaxation must be explicit in docs and tests; silent acceptance drift is not allowed.

### GRAM-05 Horizontal truncation is forbidden, not tolerated

- Requirement
  - `...[N chars omitted]` and any equivalent horizontally truncated line representation are invalid in `apply_splice` selections.
- Pass checks
  - Selection validation fails when any selected line contains a horizontal truncation marker.
  - Failure occurs at validation time before any writes.
  - The error code family distinguishes truncation from ordinary content mismatch.
- Required evidence
  - `E1`: explicit prohibition in spec/tool docs.
  - `E2`: resolver tests with sampled-view-derived truncated lines.
  - `E3`: diagnostics assertion that the error points to the offending selection line.
  - `E5`: regression test sourced from a real `read_file` sampled-view fixture.
- Negative-test expectations
  - Reject a line like `42 | prefix...[18 chars omitted]` even if the file contents would otherwise match.
- Regression requirement
  - Shared numbered-excerpt helpers must expose a strict mode that rejects truncation; lenient sampled-view behavior must not leak into splice.

### GRAM-06 Parse and validation errors are no-write failures

- Requirement
  - Any grammar or authored-surface well-formedness failure must terminate before commit planning performs writes.
- Pass checks
  - Filesystem state is unchanged after malformed input.
  - No partial output files or secondary diagnostic writes occur.
- Required evidence
  - `E2`: parser/validation tests with before/after filesystem assertions.
  - `E5`: permanent no-write regression for malformed programs.
- Negative-test expectations
  - Malformed second action in a multi-action program must not mutate files from the first action if the parser/validator cannot produce a valid program.
- Regression requirement
  - Future parser refactors must preserve parse-before-execute discipline.

## 4.2 Selection Semantics

### SEL-01 Source and target selection use double-lock validation

- Requirement
  - A selection resolves only if both the absolute line numbers and the visible content on those numbered lines match the target snapshot.
- Pass checks
  - Matching numbers with wrong content fail.
  - Matching content at wrong line numbers fails.
  - Matching both succeeds.
- Required evidence
  - `E1`: double-lock rule in stable spec.
  - `E2`: resolver tests for line-number drift, content drift, and exact match.
  - `E3`: diagnostics tests proving source mismatch blames source selection and target mismatch blames target selection.
  - `E5`: regression tests for both drift classes.
- Negative-test expectations
  - If a block moved elsewhere in the file, the original line-number reference must fail even if the visible content still exists somewhere else.
- Regression requirement
  - Fuzzy or semantic matching must remain out of scope for v1.

### SEL-02 Selection denotation is contiguous, never sparse

- Requirement
  - A validated selection denotes exactly one contiguous line interval `[a, b]`.
  - Omission compresses interior lines; it does not skip them semantically.
- Pass checks
  - `10 | alpha` then omission then `14 | omega` denotes `10..14` inclusive.
  - `10 | alpha` then `14 | omega` without omission fails.
- Required evidence
  - `E1`: formal validation relation and contiguous-interval wording.
  - `E2`: resolver tests that reconstruct intervals and prove included interior lines.
  - `E5`: regression tests for non-contiguous numbering without omission.
- Negative-test expectations
  - Reject sparse-sample interpretations such as selecting lines 10 and 14 only.
- Regression requirement
  - Any optimization in the resolver must preserve contiguous-range denotation.

### SEL-03 Numbering monotonicity and omission legality are enforced by the resolver

- Requirement
  - Numbered lines are strictly increasing.
  - Duplicate or descending numbers are invalid.
  - Omission legality is checked against actual numeric gaps.
- Pass checks
  - Duplicate number fails.
  - Descending number fails.
  - Omission over empty or adjacent gap fails.
  - Numeric gap without omission fails.
- Required evidence
  - `E2`: dedicated resolver tests for each invalid form.
  - `E3`: diagnostics assertions on the exact offending selection item.
  - `E5`: boundary regressions for smallest failing gaps.
- Negative-test expectations
  - Reject `10`, `12`, omission, `13`.
  - Reject `10`, omission, `11`.
- Regression requirement
  - Error classification must distinguish malformed selection structure from snapshot drift.

### SEL-04 Anchored target modes operate on ranges, not single anchor lines

- Requirement
  - `Insert Before` inserts at the byte boundary before the denoted target range.
  - `Insert After` inserts at the byte boundary after the denoted target range.
  - `Replace` replaces the full denoted target range.
- Pass checks
  - Multi-line target selections are accepted and interpreted as ranges.
  - Insert-before and insert-after around a multi-line target range produce distinct, correct byte positions.
  - Replace over a multi-line target range removes the entire target span.
- Required evidence
  - `E1`: formal target-range semantics.
  - `E2`: runtime tests with multi-line target selections for before/after/replace.
  - `E5`: regression tests covering single-line and multi-line anchored targets.
- Negative-test expectations
  - Reject implementations that treat a multi-line target selection as “first line only” or “last line only”.
- Regression requirement
  - Boundary computation must remain defined in byte offsets, not reconstructed logical lines.

### SEL-05 Target existence semantics stay narrow

- Requirement
  - `Append To File` may create a missing destination file.
  - `Insert Before`, `Insert After`, and `Replace` require the target file and target selection to already exist.
- Pass checks
  - Append to missing file succeeds and creates the file.
  - Insert/replace on missing target fails during planning.
  - Anchored actions on existing file with non-matching target selection fail as target-selection invalid, not as write error.
- Required evidence
  - `E1`: target-existence rules in spec/tool docs.
  - `E2`: runtime tests for missing and present target cases.
  - `E3`: diagnostics tests distinguishing planning failure from write failure.
  - `E5`: regression coverage for append-create vs anchored-fail split.
- Negative-test expectations
  - Reject any implementation that fabricates empty target context for insert/replace.
- Regression requirement
  - Any future broadening of target creation rules must be treated as product-scope expansion, not an incidental runtime tweak.

### SEL-06 Source selections always denote existing bytes, never authored or reconstructed text

- Requirement
  - The resolver must only bind to bytes already present in the source snapshot.
  - No normalization, templating, or textual reconstruction is allowed between selection validation and byte-span capture.
- Pass checks
  - Raw-byte read of the resolved span equals the bytes later transferred.
  - Empty lines, whitespace-only lines, and lines with mixed indentation resolve without alteration.
- Required evidence
  - `E2`: resolver/runtime tests using `fs::read`, not only `read_to_string`.
  - `E5`: raw-byte regression fixtures with tabs, spaces, and empty lines.
- Negative-test expectations
  - Reject any implementation that trims trailing spaces or re-serializes the selected block before transfer.
- Regression requirement
  - Byte-capture logic must remain separately testable from presentation code.

## 4.3 Same-File Semantics

### SAME-01 All same-file selections resolve against one original snapshot

- Requirement
  - For every commit unit, all source and target selections touching the same file resolve against the file's pre-mutation snapshot.
- Pass checks
  - Multi-action same-file programs validate against `F0`, not against intermediate mutations.
  - Reordering internal execution mechanics without changing authored order does not change selection meaning.
- Required evidence
  - `E1`: original-snapshot rule in formal semantics/spec.
  - `E2`: runtime-planning tests that capture one original snapshot and validate all same-file selections against it.
  - `E5`: regression with two same-file actions where the second would only succeed under intermediate-state interpretation.
- Negative-test expectations
  - A later same-file selection must fail if it refers to text inserted by an earlier action in the same program.
- Regression requirement
  - Snapshot capture must happen before any same-file offset translation or mutation staging.

### SAME-02 Intermediate-state-dependent interpretation is explicitly rejected

- Requirement
  - The runtime must reject authored programs whose meaning depends on reading the mutated result of a previous action in the same commit unit.
- Pass checks
  - The documented counterexample class fails with a semantic diagnostic.
  - Failure is attributed to the authored selection or action causing the invalid interpretation, not to a later write-stage symptom.
- Required evidence
  - `E1`: explicit prohibition in semantics/spec.
  - `E2`: dedicated negative runtime tests reproducing intermediate-state dependence.
  - `E3`: diagnostics assertions for truthful blame location.
  - `E5`: stable regression fixture for this class.
- Negative-test expectations
  - Reject “insert block, then replace the inserted block by referring to its new line numbers” in the same file.
- Regression requirement
  - Planner logic must not silently reinterpret authored line numbers after a prior in-memory mutation.

### SAME-03 Same-file overlap is illegal in v1

- Requirement
  - For same-file `Insert Before`, `Insert After`, and `Replace`, overlap between the source interval and target range is a hard planning error.
- Pass checks
  - Overlap is detected before writes.
  - Source and target file contents remain unchanged after rejection.
  - Diagnostic uses a stable overlap-class code and blames the action header.
- Required evidence
  - `E1`: overlap prohibition in spec.
  - `E2`: runtime tests for copy and move overlap across all three anchored target modes.
  - `E3`: presentation tests for overlap error code and blame location.
  - `E5`: dedicated regressions for boundary-touching non-overlap vs true overlap.
- Negative-test expectations
  - Reject partial overlap, exact overlap, and containment in either direction.
- Regression requirement
  - If overlap is ever supported in a future version, it must ship behind a documented semantic redesign, not by weakening the current checks.

### SAME-04 Non-overlapping same-file moves translate offsets correctly after source removal

- Requirement
  - For valid non-overlapping same-file moves, source and target offsets are first computed against the original snapshot, then translated only as an execution detail after source removal.
- Pass checks
  - Moving a block upward and downward in the same file yields correct final ordering.
  - Insert-before, insert-after, and replace each honor the formal translation rule.
  - No duplicated or dropped bytes occur.
- Required evidence
  - `E1`: translation rule in formal semantics.
  - `E2`: runtime tests for same-file move with source before target and source after target.
  - `E5`: byte-exact regressions for same-file move translation.
- Negative-test expectations
  - Reject any implementation that computes target insertion after source removal by re-resolving authored selections against mutated content.
- Regression requirement
  - Same-file move tests must assert raw file bytes, not only rendered line content.

### SAME-05 Same-file copies are deterministic and non-destructive

- Requirement
  - Same-file copy preserves the original source bytes in place and inserts/replaces only at the target boundary/range derived from the original snapshot.
- Pass checks
  - Source bytes remain untouched.
  - Target effect occurs exactly once.
  - Result is independent of staging implementation details.
- Required evidence
  - `E2`: runtime tests for same-file copy across append, insert-before, insert-after, and replace where legal.
  - `E5`: regression fixtures proving no accidental source deletion.
- Negative-test expectations
  - Reject any accidental move-like behavior in copy mode.
- Regression requirement
  - Copy/move mode branching must be covered by mode-pair tests, not inferred from shared helpers alone.

## 4.4 Byte Fidelity and Newline Behavior

### BYTE-01 Transfer span equals the exact source byte interval

- Requirement
  - The transferred payload is exactly `Bytes(F)[start_a, end_b)` for the validated source interval.
- Pass checks
  - Runtime captures bytes by offsets, not by joining rendered lines.
  - Raw bytes transferred equal raw bytes originally present in the selected interval.
- Required evidence
  - `E1`: byte-span definition in formal semantics/spec.
  - `E2`: raw-byte unit tests that assert equality by `fs::read` and byte slices.
  - `E5`: regression with non-UTF8-safe-looking but valid byte patterns if the file API permits; otherwise mixed newline and whitespace fixtures at minimum.
- Negative-test expectations
  - Reject implementations that rebuild source text from logical lines and reinsert synthesized separators.
- Regression requirement
  - Tests must fail if separator handling changes while visible text remains the same.

### BYTE-02 Separator bytes are preserved exactly

- Requirement
  - `\n`, `\r\n`, and empty final separator state are preserved exactly from the selected source interval.
- Pass checks
  - A source block ending in `\n` transfers `\n`.
  - A source block ending in `\r\n` transfers `\r\n`.
  - A final source line without trailing newline transfers without synthesized newline.
- Required evidence
  - `E2`: raw-byte tests for all three cases.
  - `E5`: regression fixtures stored as bytes, not normalized text literals where possible.
- Negative-test expectations
  - Reject silent newline normalization to the destination file's dominant style.
- Regression requirement
  - The test suite must include at least one mixed-newline fixture and one EOF-without-newline fixture.

### BYTE-03 Mixed newline styles inside the selected source interval remain mixed

- Requirement
  - If the source interval contains mixed separators, those exact separator bytes are transferred unchanged.
- Pass checks
  - A fixture containing both `\r\n` and `\n` inside the selected span is transferred byte-for-byte.
  - Destination bytes reflect the original mixed style of the copied/moved source block.
- Required evidence
  - `E2`: runtime byte tests with mixed newline fixtures.
  - `E5`: permanent mixed-newline regression.
- Negative-test expectations
  - Reject “normalize on write” behavior even if the destination file otherwise uses a consistent style.
- Regression requirement
  - Byte-fidelity tests must compare complete file byte arrays before and after splice, not line-oriented renderings only.

### BYTE-04 Empty lines and whitespace-only lines are preserved as selected

- Requirement
  - Empty visible lines, whitespace-only lines, and lines with leading/trailing spaces remain exact in transferred bytes.
- Pass checks
  - Selection containing an empty line transfers that line and its separator correctly.
  - Whitespace-only lines are preserved without trimming or collapsing.
- Required evidence
  - `E2`: byte and semantic tests for empty and whitespace-only lines.
  - `E5`: regression fixture containing empty line plus indented and trailing-space lines.
- Negative-test expectations
  - Reject any implementation that canonicalizes blank/whitespace-only lines during parse or write.
- Regression requirement
  - Presentation-oriented formatting helpers must never feed back into byte transfer logic.

### BYTE-05 Move semantics do not corrupt source-adjacent bytes

- Requirement
  - Removing a moved source span must leave untouched bytes outside `[start_a, end_b)` byte-identical to the pre-mutation snapshot, except where another validated action in the same unit intentionally modifies them.
- Pass checks
  - Prefix and suffix bytes around a moved span remain unchanged.
  - Adjacent separator behavior is correct for moved final lines and moved interior lines.
- Required evidence
  - `E2`: raw-byte runtime tests for moving first, middle, and last ranges.
  - `E5`: regression fixtures around source-boundary edge cases.
- Negative-test expectations
  - Reject accidental deletion of the wrong separator or accidental duplication of a neighboring separator.
- Regression requirement
  - Boundary tests must cover move-out-of-middle and move-final-line-without-newline cases.

## 4.5 Atomicity, Commit Units, and Outcome Semantics

### ATOM-01 Connected commit units are grouped by path identity, not string spelling alone

- Requirement
  - Actions touching aliased forms of the same path must belong to the same commit unit.
  - Disjoint path sets may become separate commit units.
- Pass checks
  - Alias paths such as `sub/../item.txt` and `item.txt` group into one unit.
  - On Windows, case aliases group correctly if the shared substrate already does so.
- Required evidence
  - `E1`: architecture or runtime design text naming shared path-identity substrate.
  - `E2`: shared-substrate tests modeled after current patch alias-grouping coverage.
  - `E5`: regression tests for normalized alias grouping.
- Negative-test expectations
  - Reject any planner that allows one alias form to commit while another alias form in the same semantic file fails separately.
- Regression requirement
  - Alias-grouping tests must live below the splice runtime when the behavior is shared substrate.

### ATOM-02 Every commit unit is planned completely before writes begin

- Requirement
  - Selection validation, overlap checks, target existence checks, and per-path post-state computation for a unit complete before any writes for that unit begin.
- Pass checks
  - Planning errors leave all paths in the unit unchanged.
  - Write stage begins only after the full unit has a valid plan.
- Required evidence
  - `E1`: planning-and-commit phase description.
  - `E2`: runtime tests where a late planning failure would otherwise have allowed partial writes.
  - `E5`: no-partial-write regression for multi-path move/copy units.
- Negative-test expectations
  - A move whose target selection is invalid must not remove source bytes first and “discover” failure later.
- Regression requirement
  - Planner and committer must remain separable in tests and code review.

### ATOM-03 Unit commit is atomic under write failure

- Requirement
  - If any write in a commit unit fails, all earlier writes in that unit are rolled back and the unit is reported as failed.
- Pass checks
  - Simulated write failure after one target path is written results in full rollback of the unit.
  - No half-move state is observable.
  - Parent-directory creation for append-to-missing-file occurs during commit and participates in rollback discipline for file contents.
- Required evidence
  - `E2`: runtime/shared-substrate tests forcing write failure during multi-path commit.
  - `E3`: failure presentation test for write-stage diagnostics.
  - `E5`: rollback regression modeled after current staged-write behavior.
- Negative-test expectations
  - Reject “destination written, source not removed” and “source removed, destination not written” states.
- Regression requirement
  - Tests must cover both cross-file move and append-create-with-parent-directory scenarios.

### ATOM-04 Program status model matches connected-unit semantics

- Requirement
  - Full success means all commit units committed.
  - Partial success means at least one unit committed and at least one unit failed.
  - Failure means no unit committed.
- Pass checks
  - Multi-unit program with one successful disjoint unit and one failing disjoint unit reports partial success.
  - Single failing unit with no commits reports failure.
  - Fully successful multi-unit program reports full success.
- Required evidence
  - `E1`: status model in formal semantics.
  - `E2`: runtime tests for all three status outcomes.
  - `E3`: presentation assertions for partial-success rendering.
  - `E5`: permanent regressions for status classification.
- Negative-test expectations
  - Reject any implementation that rolls back an already committed disjoint unit because a later disjoint unit failed.
- Regression requirement
  - Status tests must assert both result classification and actual filesystem state.

### ATOM-05 Outcome summaries remain family-compatible and coarse

- Requirement
  - Success output uses compact `A/M/D` path summaries.
  - Cross-file move-shaped success normalizes to destination-side `M`.
  - Net-zero results do not fabricate affected entries.
- Pass checks
  - Copy to missing destination yields `A dest`.
  - Copy to existing destination yields `M dest`.
  - Same-file copy and same-file move yield `M file`.
  - Cross-file move yields destination-side `M`, not `D source` plus `A dest` in the visible success block.
  - Net-zero composition yields no affected entry.
- Required evidence
  - `E1`: success-summary rules in tool docs/spec.
  - `E2`: runtime tests for affected-path classification.
  - `E3`: presentation tests for rendered success text.
  - `E4`: CLI/MCP parity tests for summary text.
  - `E5`: regression tied to the destination-side `M` baseline.
- Negative-test expectations
  - Reject stale summary behavior that reports `M from.txt` for cross-file move success.
- Regression requirement
  - Any future expansion of visible success detail must preserve the coarse family-compatible summary as the default path.

### ATOM-06 The full current action basis is implemented and tested

- Requirement
  - The current contract includes eight transfer actions from `Copy/Move × Append/Insert Before/Insert After/Replace` plus one source-only `Delete Span` primitive, with independent behavioral proof for each action.
- Pass checks
  - Each action has at least one passing semantic test.
  - Each action has at least one nearest-boundary negative test.
  - Each action has expected affected-path classification.
- Required evidence
  - `E2`: explicit scenario tests covering all transfer combinations plus `Delete Span`.
  - `E5`: durable scenario matrix entry for each action.
- Negative-test expectations
  - No action may be “accepted by parser but intentionally unimplemented” at release time.
- Regression requirement
  - Coverage reports or test inventory must make the full action basis reviewable without manual guesswork.

## 4.6 Diagnostics and Failure Surface

### DIAG-01 Stable minimal diagnostic family is present and reviewable

- Requirement
  - The implementation exposes a stable minimum diagnostic family covering:
    - `SPLICE_SOURCE_SELECTION_INVALID`
    - `SPLICE_TARGET_SELECTION_INVALID`
    - `SPLICE_SELECTION_TRUNCATED`
    - `SPLICE_OVERLAP_ILLEGAL`
    - `SPLICE_TARGET_STATE_INVALID`
    - `SPLICE_WRITE_ERROR`
    - `SPLICE_PARTIAL_UNIT_FAILURE`
- Pass checks
  - Each code can be triggered by at least one automated test.
  - The code names used in docs, output, and tests match exactly.
- Required evidence
  - `E1`: diagnostics contract in stable docs or accepted closure artifact.
  - `E3`: presentation tests for every minimum code.
  - `E5`: one regression per code family.
- Negative-test expectations
  - Reject collapsing all failures into one generic splice error.
- Regression requirement
  - Renaming or splitting codes requires coordinated docs and tests updates in the same change.

### DIAG-02 Blame hierarchy is truthful and stable

- Requirement
  - Source mismatch blames the source selection block.
  - Target mismatch blames the target selection block.
  - Overlap or semantic ordering errors blame the action header.
  - Write failures blame the target path or destination anchor when no stronger authored span exists.
- Pass checks
  - Automated tests assert line/column or equivalent rendered anchor for each class.
  - No failure invents a more precise authored span than the runtime actually knows.
- Required evidence
  - `E1`: blame hierarchy in spec.
  - `E3`: presentation tests for each blame class.
  - `E4`: transport tests proving the same blame location reaches both MCP and CLI.
  - `E5`: regression on at least one source-mismatch, one target-mismatch, one overlap, and one write-error case.
- Negative-test expectations
  - Reject reporting a target mismatch on the source block.
  - Reject fabricating a precise selection-line blame for a raw OS write error.
- Regression requirement
  - New diagnostics helpers must preserve honesty over prettiness.

### DIAG-03 Failure text starts compact and remains repair-oriented

- Requirement
  - Failures begin with `error[CODE]: ...` and keep the inline text as the primary repair surface.
  - Optional `help:` lines are allowed and should be actionable.
- Pass checks
  - Failure output includes code, summary, authored or target location, and concise next-step guidance where appropriate.
  - Output does not bury the primary failure behind sidecar references.
- Required evidence
  - `E1`: diagnostics philosophy alignment.
  - `E3`: golden-output tests for representative failure classes.
  - `E4`: CLI/MCP parity test for full rendered text.
  - `E5`: regression on the exact headline format.
- Negative-test expectations
  - Reject tutorial-style verbosity that obscures the actionable failure.
  - Reject outputs that require opening a sidecar to understand the basic failure.
- Regression requirement
  - Headline format and help-line conventions must remain under snapshot tests.

### DIAG-04 Optional target anchors are evidence-based, not speculative

- Requirement
  - A target anchor may be shown only when the runtime has strong corroborating target-side evidence.
  - Absence of target-anchor evidence is acceptable; fabrication is not.
- Pass checks
  - Tests cover one failure with target anchor and one without.
  - Inline diagnostics preserve target-anchor evidence only when truly supported.
- Required evidence
  - `E3`: presentation tests for anchored and unanchored failure rendering.
  - `E5`: regression ensuring weak-evidence cases do not emit anchors.
- Negative-test expectations
  - Reject always-on target anchors.
- Regression requirement
  - Anchor-emission heuristics must be directly testable and not hidden inside string-formatting code.

### DIAG-05 Partial-success diagnostics preserve both committed and failed information

- Requirement
  - Partial-success output must include committed `A/M/D`, failed units/groups, attempted `A/M/D` for failed units, and repair-oriented help.
- Pass checks
  - Partial failure renders both what succeeded and what failed.
  - The user can identify the committed subset without opening sidecars.
  - The user can identify the failed subset and retry strategy.
- Required evidence
- `E1`: partial-success repair-accounting requirement in docs.
  - `E3`: golden-output tests for partial success.
  - `E4`: MCP/CLI parity tests for partial-success output.
  - `E5`: regression modeled after current patch partial-failure behavior.
- Negative-test expectations
  - Reject partial failure outputs that hide committed changes.
  - Reject outputs that omit attempted changes for failed units.
- Regression requirement
  - Large-list summarization must preserve the same meaning when inline output is abbreviated.

### DIAG-06 Splice failure handling remains repair-first and does not introduce audit-shaped sidecars

- Requirement
  - Splice diagnostics remain inline-first in visible output.
  - If splice later needs source persistence for model repair, that persistence must remain a repair object rather than an audit/report subsystem.
  - Durable audit trails, if needed, belong to the Codex host rather than tool-managed sidecars.
- Pass checks
  - No splice-specific sidecar schema is required to understand or repair failures.
  - Inline output contains the same primary failure semantics directly.
- Required evidence
  - `E1`: host-audit boundary documented.
  - `E3`: tests proving failure handling does not depend on tool-managed sidecars.
  - `E4`: server / CLI parity tests for the repair-first diagnostics contract.
  - `E5`: regression on absence of tool-emitted sidecar requirements.
- Negative-test expectations
  - Reject introducing patch-style sidecar schemas as part of splice's public contract.
- Regression requirement
  - Any future audit-related behavior must remain host-owned unless the product direction is explicitly reopened.

## 4.7 MCP / CLI Parity

### PAR-01 Success text parity is byte-for-byte stable across MCP and CLI

- Requirement
  - Given equivalent workspace context and the same splice program, CLI stdout and MCP text output must match exactly for success responses.
- Pass checks
  - One successful splice scenario is asserted end-to-end in both transports with exact text equality.
  - Resulting filesystem state matches in both transports.
- Required evidence
  - `E4`: parity tests modeled after existing patch parity tests.
  - `E5`: permanent regression for a successful splice.
- Negative-test expectations
  - Reject transport-specific rewording of success summaries.
- Regression requirement
  - CLI must continue reading splice programs from stdin if that is the chosen UX shape for patch parity.

### PAR-02 Failure text parity is stable across MCP and CLI

- Requirement
  - Given equivalent workspace context and the same failing splice program, CLI and MCP must surface the same failure text.
- Pass checks
  - Error code, headline, blame location, help text, committed changes, and failed-group sections match.
- Required evidence
  - `E4`: parity tests for at least one planning failure and one partial-success failure.
  - `E5`: regressions for both cases.
- Negative-test expectations
  - Reject CLI-only or MCP-only diagnostic detail.
- Regression requirement
  - Failure parity tests must not depend on stripping transport-specific post-processing, because no tool-emitted artifact notes should exist.

### PAR-03 Workspace and path anchoring semantics are the same across transports

- Requirement
  - Relative paths resolve against the active workspace.
  - Absolute paths remain absolute.
  - Missing workspace for relative paths fails consistently across MCP and CLI.
- Pass checks
  - Equivalent workspace setup yields identical path interpretation.
  - Missing-workspace cases fail with equivalent guidance.
- Required evidence
  - `E1`: tool docs/spec mention path rules.
  - `E4`: transport tests for relative path success and missing-workspace failure.
  - `E5`: regression on anchoring behavior.
- Negative-test expectations
  - Reject CLI-only implicit path anchoring that is not mirrored in MCP semantics.
- Regression requirement
  - If splice gains special anchoring behavior, it must be documented explicitly and tested in both transports.

### PAR-04 Host-audit boundary is transport-consistent

- Requirement
  - Neither transport requires tool-managed failure artifacts as part of the public contract.
  - Any audit trail expectations are delegated consistently to the host layer.
- Pass checks
  - Inline output does not mention tool-managed artifacts.
  - CLI and MCP expose the same repair-first contract.
- Required evidence
  - `E4`: server and CLI tests for absence of tool-managed artifact requirements.
  - `E5`: regression on transport parity with repair-first diagnostics.
- Negative-test expectations
  - Reject MCP-only or CLI-only sidecar behavior creeping back into the contract.
- Regression requirement
  - Artifact-writing thresholds and summarization rules must remain centrally testable.

### PAR-05 Warning behavior, if any, remains family-consistent across transports

- Requirement
  - If `apply_splice` introduces warnings on successful execution, they must append to the normal success shape rather than replace it, and they must render the same in MCP and CLI.
- Pass checks
  - Success headline and `A/M/D` summary remain primary.
  - Warning blocks follow the same rendered structure in both transports.
- Required evidence
  - `E3`: presentation tests if warnings exist.
  - `E4`: parity tests if warnings exist.
- Negative-test expectations
  - Reject transport-specific warning ordering or suppression.
- Regression requirement
  - If v1 ships with no warnings, the absence should be explicit in review notes rather than assumed.

## 4.8 Documentation Sync and Source of Truth

### DOC-01 Stable docs are updated before the feature is called complete

- Requirement
  - Before `apply_splice` is accepted as complete, stable docs must reflect the implemented contract.
- Pass checks
  - `docs/apply_splice_spec.md` is updated or explicitly superseded.
  - `docutouch-server/tool_docs/apply_splice.md` exists and matches behavior.
  - `docs/README.md` points to the correct stable source-of-truth doc.
- Required evidence
  - `E1`: doc diff in the implementation workstream.
  - `E5`: review checklist item requiring doc presence before merge.
- Negative-test expectations
  - Reject “temporary closure docs only” as the long-term contract after implementation lands.
- Regression requirement
  - Doc promotion must happen before or with implementation, not as undefined later cleanup.

### DOC-02 Source-of-truth precedence is explicit and followed

- Requirement
  - After implementation exists, precedence is: code plus passing tests, then stable spec docs, then tool docs/examples, then temporary notes/chat residue.
- Pass checks
  - Review notes explicitly name the precedence rule.
  - Drift between examples and runtime is resolved, not hand-waved.
- Required evidence
  - `E1`: source-of-truth policy recorded in stable or accepted temporary docs.
  - `E5`: release review note confirming no unresolved drift remains.
- Negative-test expectations
  - Reject quoting a stale example as authoritative against passing runtime tests.
- Regression requirement
  - Behavior-changing PRs must include either doc updates or an explicit documented decision that the current docs were already correct.

### DOC-03 Behavior changes require synchronized doc updates

- Requirement
  - Any change to tool boundary, grammar, diagnostics, success summary, or CLI/MCP semantics must update the relevant docs in the same workstream.
- Pass checks
  - Review diff shows code/test/doc changes together for contract changes.
  - Maintainer guidance remains accurate.
- Required evidence
  - `E1`: doc diff for any contract-affecting code change.
  - `E5`: review checklist confirmation.
- Negative-test expectations
  - Reject “we will update docs later” for contract-level changes.
- Regression requirement
  - Release checklist must include explicit doc-sync verification.

### DOC-04 Examples are executable or directly test-backed

- Requirement
  - Any example included in tool docs or stable docs must match a tested behavior class.
- Pass checks
  - Every example maps to at least one automated test or validated golden fixture.
  - No example contradicts outcome-summary or diagnostics baselines.
- Required evidence
  - `E1`: examples included in docs.
  - `E5`: example-to-test mapping in review notes or inline comments.
- Negative-test expectations
  - Reject stale examples that still show obsolete move summary behavior or obsolete omission tokens.
- Regression requirement
  - Example drift found in review blocks release until resolved.

## 5. Negative-Test Expectations by Release Gate

The release candidate is not acceptable unless the negative suite includes, at minimum, all of the following classes.

1. malformed envelope and malformed header spellings
2. invalid action shapes
3. malformed selection-line delimiters and non-positive line numbers
4. duplicate or descending numbering
5. omission misuse: bare `...`, wrong token kind, leading/trailing omission, adjacent omission, empty-gap omission, missing omission over real gap
6. horizontal truncation markers inside selections
7. source mismatch and target mismatch as separate failure classes
8. insert/replace against missing target file
9. same-file intermediate-state dependence
10. same-file overlap for copy and move across before/after/replace
11. same-file non-overlapping move translation edge cases
12. write-stage rollback failure during multi-path commit
13. alias-path grouping and atomic failure across aliases
14. partial success across disjoint commit units
15. host-audit-boundary and repair-first diagnostics checks
16. MCP/CLI parity failures for both success and failure

A release candidate fails this document if any one of these classes is untested or only manually spot-checked.

## 6. Regression Requirements

The permanent regression suite for `apply_splice` must satisfy all of the following.

1. Every accepted semantic rule has at least one positive test and one nearest-boundary negative test.
2. Every user-visible failure class has at least one diagnostics snapshot or structured rendering assertion.
3. Every transport-visible behavior change has at least one MCP/CLI parity assertion.
4. The full action basis is reviewable as an explicit inventory, not inferred from scattered tests.
5. Byte-fidelity coverage uses raw-byte assertions in addition to line-oriented textual assertions.
6. Partial-success coverage asserts both rendered output and resulting filesystem state.
7. Alias-path and rollback coverage live at the shared-substrate layer if that logic is extracted.
8. Example snippets included in docs are traceable to tests.
9. Regression tests must run in normal CI for the Rust workspace; they cannot live only in ad hoc local scripts.
10. If a future refactor moves logic between crates/modules, the test ownership may move, but the coverage obligations may not disappear.

## 7. Definition of Done

`apply_splice` is done only when all of the following are true.

1. All top-level acceptance goals in Section 3 are satisfied.
2. Every criterion in Section 4 has explicit pass evidence.
3. The negative suite in Section 5 exists and passes.
4. The regression obligations in Section 6 are met in CI.
5. Stable docs and tool docs are updated and consistent with runtime behavior.
6. MCP and CLI parity tests pass for representative success, planning failure, write failure, and partial-success cases.
7. The release diff shows no unresolved drift between examples, docs, and tests.
8. Review can explain any remaining v1 exclusions as intentional product boundaries rather than accidental gaps.

If any item above is false, `apply_splice` is not ready to count as accepted.

## 8. Reviewer Walkthrough Checklist

Use this checklist literally during review. Every unchecked box blocks acceptance.

- [ ] `apply_splice` still has a narrow identity: transfer existing bytes only, no inline authored text.
- [ ] The accepted grammar is exact and documented: envelope, action headers, target headers, and selection syntax.
- [ ] Parser tests exist for every accepted header and every important near-miss rejection.
- [ ] Action-shape validation rejects missing/extra source or target selection blocks, including delete-with-target shapes.
- [ ] Source selections require exact source omission tokens; target selections require exact target omission tokens.
- [ ] Bare `...` is rejected in splice selections.
- [ ] Horizontal truncation markers are rejected with a dedicated truncation-class diagnostic.
- [ ] Selection tests prove double-lock validation: wrong content fails, wrong line number fails, exact match passes.
- [ ] Selection tests prove omission denotes a contiguous range, not sparse sampling.
- [ ] Target-range tests prove `Insert Before`, `Insert After`, and `Replace` operate on full target ranges.
- [ ] Append-to-missing-file succeeds; insert/replace on missing target fails in planning.
- [ ] Same-file tests prove all selections resolve against one original snapshot.
- [ ] Same-file tests prove intermediate-state-dependent interpretation is rejected.
- [ ] Same-file overlap tests exist for copy and move across before/after/replace.
- [ ] Non-overlapping same-file move translation is covered for both source-before-target and source-after-target cases.
- [ ] Byte-fidelity tests use raw-byte assertions, not only rendered strings.
- [ ] Byte-fidelity tests cover `\n`, `\r\n`, mixed separators, EOF-without-newline, empty lines, and whitespace-only lines.
- [ ] Move tests prove source-adjacent bytes are not corrupted during removal.
- [ ] Commit-unit tests prove alias paths group atomically.
- [ ] Planning failures leave the whole affected unit unchanged.
- [ ] Write failures roll back the whole unit with no half-move state.
- [ ] Program status tests cover full success, partial success, and failure.
- [ ] Success summaries use family-compatible `A/M/D` output.
- [ ] Cross-file move success normalizes to destination-side `M`.
- [ ] Net-zero effects do not fabricate affected entries.
- [ ] All transfer combinations plus `Delete Span` have explicit positive coverage.
- [ ] All minimum splice diagnostic codes exist and are test-covered.
- [ ] Blame hierarchy is test-covered for source mismatch, target mismatch, overlap, and write failure.
- [ ] Failure output starts with `error[CODE]:` and remains repair-oriented.
- [ ] Optional target anchors appear only when supported by strong evidence.
- [ ] Partial-success output includes committed changes, failed groups, attempted changes, and retry guidance.
- [ ] No audit-shaped tool-managed sidecars or patch-run caches are emitted.
- [ ] CLI success output matches MCP success output for the same scenario.
- [ ] CLI failure output matches MCP failure output without transport-specific sidecar notes or artifact paths.
- [ ] Workspace/path anchoring rules are the same in MCP and CLI.
- [ ] Repair-first failure behavior and host-owned audit boundary are the same in MCP and CLI.
- [ ] Stable docs are updated: spec, tool docs, and docs index.
- [ ] Examples in docs are aligned with tested runtime behavior.
- [ ] No known drift remains between code, tests, output, and docs.
- [ ] CI includes the new parser, resolver, runtime, presentation, and transport tests.
- [ ] Remaining v1 exclusions are explicitly documented rather than accidentally unimplemented.
