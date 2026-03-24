# apply_patch Semantics, UX, and Hardening Plan

## Status

- Drafted after black-box testing in `temporary/patch-tool-blackbox-20260319`
- Reviewed against the local DocuTouch fork and the public OpenAI `openai/codex` repository
- Originally written as the working design record before code changes began
- Partially implemented as of 2026-03-20:
  - compatibility notes landed in the tool docs
  - success-path overwrite warnings landed
  - partial/failure UX now preserves committed `A/M/D` and enumerates failed file groups with structured diagnostics
  - malformed `Add File` lines now report a more precise outer-format error
  - net-zero patch updates are confirmed to elide filesystem writes instead of touching timestamps

## Why this document exists

Two separate questions were initially mixed together:

1. Should DocuTouch tighten `Add File` and `Move to` into stricter semantics?
2. How should the tool communicate the current runtime behavior to its only consumer, the LLM itself?

The current conclusion is deliberately split:

- do not immediately hard-tighten general `Add File` overwrite and `Move to` overwrite behavior
- do document the behavior clearly but without teaching it as a preferred tactic
- do add triggered warnings when these special behaviors actually occur
- do prioritize correctness hardening for path identity, Windows path edge cases, workspace/path correctness, and same-path move bugs

## Current behavior summary

### Upstream-style authoring contract

The public OpenAI tool instructions describe `Add File` as creating a new file and frame patch authoring in a relatively strict way.

Relevant upstream reference:

- `openai/codex` -> `codex-rs/apply-patch/apply_patch_tool_instructions.md`

### Runtime behavior already tolerated upstream

Public upstream fixtures explicitly codify the following behaviors:

- `011_add_overwrites_existing_file`
- `010_move_overwrites_existing_destination`

This means the broader runtime behavior is not accidental. It is part of the compatibility surface that the public OpenAI repository currently tests.

### Local DocuTouch behavior

The local DocuTouch fork keeps upstream parser and matching behavior close to baseline, while intentionally strengthening only the file-group commit model.

Current local behavior confirmed by black-box testing:

- `Add File` over an existing file replaces that file's contents
- `Move to` over an existing destination replaces the destination contents
- repeated `Update File` blocks on the same existing path are applied in order
- `Add File` followed by `Update File` on the same path works inside one DocuTouch commit unit
- independent file groups may partially succeed
- connected file groups remain atomic
- partial-failure rendering now preserves committed `A/M/D` changes and lists failed file groups separately
- a net-zero `Update File` patch may succeed with no affected files and no filesystem timestamp churn

## Product decision record

### Decision D1: do not hard-tighten general Add/Move overwrite semantics yet

Reasoning:

- public upstream fixtures already encode these behaviors
- these behaviors are likely inside the practical model/tool-use distribution
- a default hard break would create friction against upstream compatibility and agent expectations
- there is no public upstream maintainer note saying these semantics should be rejected globally

This is not a claim that the behaviors are ideal. It is a compatibility-aware decision.

### Decision D2: document the behavior, but as a compatibility note, not as a recommendation

Reasoning:

- the tool contract should not silently differ from reality
- the documentation should not encourage the model to choose these patterns on purpose
- the preferred authoring guidance should remain narrow even if the runtime is tolerant

### Decision D3: add triggered warnings in successful UX when special behaviors occur

Reasoning:

- silent success reinforces the wrong mental model
- docs alone are too passive
- warnings should only appear when the special behavior actually happened
- the warning text must state what happened and what is preferred, without turning the behavior into a technique tutorial

### Decision D4: prioritize correctness hardening for path identity and Windows semantics

Reasoning:

- path aliasing can break file-group boundaries and semantic checks
- Windows case-insensitive paths make lexical equality insufficient
- same-path or alias-path move is a correctness hazard
- workspace/path correctness issues are clearer bugs than the general Add/Move overwrite tolerance

### Decision D5: preserve the Codex success summary shape and append warnings as independent diagnostic blocks

Reasoning:

- the most common success path should stay maximally aligned with the Codex/core distribution
- the built-in success summary already carries high-value `A/M/D` operation signals
- replacing that summary with a DocuTouch-specific success header weakens inline operation accounting and loses operation type precision
- triggered warnings are still valuable, but they should be rendered as warning blocks, not by rewriting the primary success contract
- this keeps the common path boring and stable while still allowing strategy-shaping feedback when special compatibility behavior occurs

## Documentation design

### Target file

- `docutouch-server/tool_docs/apply_patch.md`

### Intended change

Add a short section near execution semantics or failure surface. The section should be factual, compact, and low-prominence.

Suggested section title:

- `Compatibility Notes`

Suggested content shape:

- `Add File` is intended for creating a new file.
- In the current runtime, if the path already exists as a file, the file contents are replaced.
- `Move to` is intended for renaming to a destination path.
- In the current runtime, if the destination already exists as a file, the destination contents are replaced.
- Prefer `Update File` when editing an existing file.
- Prefer moving to a fresh destination path when renaming.

### Documentation principles

Do:

- say what the runtime currently does
- say what the preferred authoring pattern is
- keep the wording compact and factual

Do not:

- add positive examples that demonstrate overwrite-via-Add
- add positive examples that demonstrate overwrite-via-Move
- phrase the text as a recommendation or optimization tip
- imply that these special behaviors are the primary semantics

## UX warning design

### Goal

When the patch succeeds but used a risky compatibility behavior, the tool should say so in a warning-like style that matches the current rustc-inspired diagnostics tone.

### Consumer assumption

The tool consumer is always the LLM. Therefore:

- the warning should be machine-comprehensible and strategy-shaping
- it does not need consumer-facing ornamentation for humans
- it should align with the current terse diagnostic style

### Trigger conditions

Emit warnings only when the behavior actually occurred.

Initial warning candidates:

- `ADD_REPLACED_EXISTING_FILE`
- `MOVE_REPLACED_EXISTING_DESTINATION`

Possible future correctness warnings or errors:

- `PATH_ALIAS_COLLISION`
- `MOVE_DEST_SAME_AS_SOURCE`
- `WINDOWS_CASE_ALIAS_COLLISION`

### Recommended rendered shape

Preserve the core/Codex success summary as the primary success block. If warnings are present, append them as independent diagnostic blocks after a blank separator line.

No-warning success should stay aligned with the core summary shape:

```text
Success. Updated the following files:
A path/to/file
M path/to/other
```

Warning-bearing success should keep the same success summary, then append warning blocks:

```text
Success. Updated the following files:
A notes.md

warning[ADD_REPLACED_EXISTING_FILE]: Add File targeted an existing file and replaced its contents
  --> notes.md
  = help: prefer Update File when editing an existing file
```

Example for move:

```text
Success. Updated the following files:
M to.txt

warning[MOVE_REPLACED_EXISTING_DESTINATION]: Move to targeted an existing file path and replaced the destination contents
  --> to.txt
  = help: prefer a fresh destination path when renaming
```

### Why warning-on-trigger is preferred over doc-only disclosure

- the runtime behavior remains visible when it matters
- the model gets immediate feedback after the exact action that should be discouraged
- the warning teaches the preferred alternative without promoting the risky behavior as a general tactic

### Why the warning should not be embedded as a custom success format

- the success headline should not fork from the Codex/core baseline unless strictly necessary
- `warning[...]` deserves full diagnostic salience rather than being demoted into an inline suffix line
- mainstream toolchains commonly mix terse success summaries with standalone warning blocks; this is less surprising than a private all-in-one success dialect

## Internal data model for warnings

To avoid embedding detection logic directly inside `main.rs`, warnings should originate from lower layers as structured information.

Recommended shape:

- runtime detects special behavior
- runtime returns structured warnings with stable codes
- server renders them using the existing diagnostic tone

Suggested internal structures:

- `ApplyOutcomeWarning`
- fields:
  - `code`
  - `summary`
  - `target_path`
  - `related_path` when relevant
  - optional `help`

This keeps future rendering flexible without losing precision.

## Hardening work that should happen before any general strict-mode debate

### H1: path identity normalization

Current risk:

- `PathBuf` lexical equality is not enough for semantic identity
- relative aliases such as `a/../b.txt` vs `b.txt` may be treated as different paths
- Windows case-insensitive paths increase the risk further

Recommended direction:

- introduce a normalized path identity key for grouping and conflict detection
- keep separate concepts for display path and semantic path identity
- make the key work for paths that do not yet exist

### H2: Windows-specific path tests

Add explicit tests for:

- path separator variants
- case-insensitive aliases
- relative alias forms with `.` and `..`
- move behavior under Windows path identity rules
- mixed newline inputs where relevant to path and patch parsing behavior

### H3: same-path and alias-path move handling

This is a clearer correctness bug boundary than the general overwrite tolerance.

The runtime should not silently treat malformed move directives as destructive operations.

Recommended direction:

- reject semantically same source/destination moves if they are malformed or ambiguous
- or normalize them into a plain in-place update if the behavior is clearly non-destructive
- never allow a same-logical-path move to silently delete or corrupt the file

### H4: workspace/path correctness

Path resolution should remain consistent with the selected workspace and not drift into accidental edits in the wrong tree.

## Detailed implementation plan

### Phase 0: design/documentation checkpoint

Deliverables:

- this document
- short pointer from `DOCUTOUCH_ENHANCEMENTS.md`
- team agreement on warning-first approach

### Phase 1: document the compatibility behavior

Files:

- `docutouch-server/tool_docs/apply_patch.md`

Tasks:

- add a short compatibility note section
- keep authoring guidance narrow
- avoid positive examples of overwrite-via-Add or overwrite-via-Move

### Phase 2: add structured runtime warnings

Implementation note:

- Implemented.
- Success-path overwrite warnings now flow from runtime to server.
- The same implementation wave also added structured failed-unit diagnostics so the failure path can preserve committed `A/M/D`, enumerate failed groups, and carry action / hunk metadata when available.

Files:

- `docutouch-core/src/patch_runtime.rs`
- possibly `codex-apply-patch/src/lib.rs`
- `docutouch-server/src/main.rs`

Tasks:

- detect when `Add File` replaced an existing file
- detect when `Move to` replaced an existing destination file
- add a warning list to the apply outcome
- render warnings in the success path using diagnostic-style lines

### Phase 3: path identity and Windows hardening

Files:

- `codex-apply-patch/src/lib.rs`
- tests under `codex-apply-patch/tests/fixtures/scenarios`
- unit/integration tests as needed

Tasks:

- add normalized semantic path identity
- ensure commit-unit grouping uses semantic identity where required
- add Windows-aware path alias coverage
- fix same-path and alias-path move handling

### Phase 4: revisit strict policy only after observing the new UX

Possible later work:

- optional strict profile for `Add File` create-only
- optional strict profile for `Move to` destination-must-be-absent
- only after compatibility cost is better understood

## Candidate file-level changes

### `docutouch-server/tool_docs/apply_patch.md`

- add compact compatibility note text

### `docutouch-core/src/patch_runtime.rs`

- extend `ApplyOutcome` with structured warnings
- define stable warning codes

### `docutouch-server/src/main.rs`

- preserve the core/Codex success summary shape for the common path
- append warning blocks after the summary instead of replacing the success contract

### `codex-apply-patch/src/lib.rs`

- expose enough runtime metadata to know when compatibility behaviors occurred
- later: path identity and Windows hardening

### Tests

Add or update tests for:

- successful patch plus `ADD_REPLACED_EXISTING_FILE` warning
- successful patch plus `MOVE_REPLACED_EXISTING_DESTINATION` warning
- path alias normalization
- Windows case alias behavior
- same-path or alias-path move safety

## Open questions kept intentionally deferred

These are not blockers for the first code pass:

- whether warnings should also appear in structured machine-only form beyond text rendering
- whether future strict mode belongs in the lower crate or only in the server runtime
- whether overwrite-via-Add or overwrite-via-Move should ever become default errors

## Future diagnostic directions

The current execution diagnostics are materially better than the original
failure path, but they are not the end state.

The intended standard is:

> rustc-like honesty, not rustc cosplay.

For the detailed diagnostics design that expands this principle into a concrete
specification, see `docs/apply_patch_diagnostics_spec.md`.

The most credible next directions are:

- multi-span execution diagnostics when there is a robust causal chain worth
  showing
  - for example, a primary chunk location plus one additional anchoring action
    location
  - not a general license to dump many spans by default
- stronger source mapping for commit-stage failures
  - today write failures only point back into patch source when the mapping is
    robust
  - future work can continue tightening this without inventing fake locations
- clearer host-audit boundary
  - tool-layer diagnostics should remain self-contained
  - repeated failure caching and replay artifacts should not be duplicated inside
    DocuTouch when the Codex host already records tool-call receipts
  - future work should prefer better inline blame evidence over second-layer
    persistence
  - future work should also prefer compact repair accounting over audit-shaped
    repetition when the same fact is already visible inline
- more systematic warning and error taxonomy
  - especially if future hardening introduces more path-identity or
    workspace-related failure classes

The standard for all future work should remain the same:

- preserve compactness
- avoid decorative verbosity
- never trade diagnostic truth for rustc-like aesthetics

## Recommended next coding order

1. patch tool docs
2. warning data model
3. success-path warning rendering while preserving the Codex/core success summary
4. tests for warning cases
5. path identity normalization
6. Windows and alias-path tests
7. same-path move correctness fixes

## Summary

The current direction is intentionally conservative:

- keep upstream-compatible general overwrite tolerance for now
- document it carefully
- warn when it happens
- preserve `A/M/D` visibility on both the success path and the partial-failure committed-changes path
- keep net-zero patch application side-effect free
- spend the next hardening budget on path identity, Windows semantics, and correctness bugs

This preserves compatibility while improving both tool transparency and runtime safety.
