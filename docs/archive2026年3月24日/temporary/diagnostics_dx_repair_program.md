# Diagnostics DX Repair Program

## Status

- Drafted on 2026-03-22 after investigating redundant `apply_patch` diagnostics in the Rust DocuTouch fork.
- Approved in principle by the project owner as the governing execution plan for the repair effort.
- This document is the active coordination artifact for documentation audit, code changes, tests, and closeout.
- Superseded in part on 2026-03-23 by the accepted direction that keeps audit-shaped sidecars rejected but restores failed patch source persistence as part of the repair contract.
- Statements in this file that describe the repair contract as strictly inline-only should no longer be treated as live authority.

## North Star

The diagnostics system exists to help ChatGPT/Codex-class models recover from failure with the fewest wasted tokens, the fewest wasted turns, and the smallest chance of a wrong follow-up repair.

This program therefore optimizes for:

- token-efficient diagnostics
- high information density
- truthful, repair-oriented evidence
- host-owned durable audit trails
- inline tool output that is self-contained for the immediate repair loop

This program explicitly does **not** optimize for:

- decorative compiler-style verbosity
- tool-layer audit theater
- repeating the same strategy hint across multiple layers
- summary prose that paraphrases an already-visible diagnostic without adding new information

## Product Boundary

### Host-owned audit trail

Durable audit and replay belong to the Codex host, which already retains tool-call receipts.

Implications:

- the tool must not grow a second audit subsystem
- the tool must not justify extra verbosity by appealing to durable audit needs
- historical references to tool-layer audit or recovery caches must be treated as retired direction unless explicitly re-opened

### Inline repair accounting

The tool **does** still need to show enough inline accounting for the next repair step.

That inline accounting is limited to information that changes the next action, for example:

- which changes were committed
- which file group failed
- what was attempted
- the smallest truthful cause
- the minimum useful next move

Inline repair accounting is not the same as durable audit.

## Current Problem Statement

The current `apply_patch` diagnostics have a DX regression for LLM consumers:

- the same `help:` guidance can appear more than once in the same message
- the final `caused by:` summary can paraphrase the headline instead of adding information
- full failure with a single failed unit can inherit partial-failure presentation weight
- documentation currently mixes host-audit language and inline repair language in a way that can mislead future implementation work

The immediate example that triggered this program is duplicated guidance around:

`re-read the target file and regenerate the patch with fresh context`

## Working Hypothesis

The redundancy is structural, not accidental wording drift.

The likely root causes are:

- top-level failure details inherit information from the first failed unit
- presentation renders both per-group detail and top-level repair guidance without deduplication
- the renderer currently lacks a sufficiently sharp distinction between:
  - full failure with one failed unit
  - full failure with multiple failed units
  - partial success with failed units
- the term `auditability` remains overloaded across documents and risks pushing future changes toward verbosity

## Program Goals

1. Remove redundant guidance from inline diagnostics.
2. Ensure every rendered block adds distinct repair value.
3. Preserve partial-success accounting where it materially affects the next repair step.
4. Align internal specs, temporary plans, and public tool docs with the host-audit boundary.
5. Add tests that protect information density, not just block presence.

## Non-goals

- redesigning unrelated tools
- expanding the diagnostics system into a generalized tracing subsystem
- polishing incidental English before ownership and rendering rules are fixed
- changing public behavior beyond the scope needed to improve diagnostic DX

## File Scope

### Code likely to change

- `docutouch-core/src/patch_runtime.rs`
- `docutouch-core/src/patch_presentation.rs`
- possibly `codex-apply-patch/src/lib.rs`
- focused tests under `docutouch-server/tests/stdio_smoke.rs`
- focused tests in `docutouch-core/src/patch_presentation.rs`

### Documentation likely to change

- `docs/apply_patch_diagnostics_spec.md`
- `docs/ux_hardening_plan.md`
- `docs/apply_patch_semantics_plan.md`
- `docs/cli_adapter_spec.md`
- `docs/roadmap.md`
- `docutouch-server/tool_docs/apply_patch.md`
- `docutouch-server/tool_docs/apply_patch_v1.md`
- `docutouch-server/tool_docs/apply_patch_v2.md`
- `docutouch-server/tool_docs/apply_patch_v3.md`
- `docutouch-server/tool_docs/apply_patch_v4.md`
- temporary planning docs that still risk being read as live guidance

## Execution Principles

### Principle 1: One owner per fact

Every major fact in a failure message should have one primary owner:

- headline owns stable failure classification
- source span owns blame localization
- failed unit detail owns local attempted-change evidence
- `caused by:` owns the most specific useful cause
- `help:` owns the minimum next repair move

If the same fact is already owned by one layer, another layer should not restate it unless new information is introduced.

### Principle 2: The common path stays boring

Single full failure should not be rendered with partial-failure bulk unless doing so adds clear repair value.

### Principle 3: No paraphrase without gain

If a lower-level message is already specific and truthful, the renderer should not replace it with a more generic sentence that merely restates the failure class.

### Principle 4: Durable audit is not an excuse for more text

Any requirement that sounds like “keep this in case someone needs an audit trail later” should be rejected at the tool layer unless it changes the immediate repair loop.

## Required Design Decisions

These design decisions were the key open questions for this program and have now
been resolved in the promoted spec and closeout materials:

1. Single full failure should stay compact by default.
2. Top-level `help:` should not duplicate equivalent failed-unit guidance.
3. `FailureDetails` should not mirror failed-unit `help`; per-unit guidance remains owned by `failed_units`.
4. `caused by:` should prefer specific low-level detail when it is already readable enough.
5. Historical documents should remain available, but live-contract precedence must be unmistakable.

For the live authority, prefer:

- `docs/diagnostics_polish_spec.md`
- `docs/temporary/diagnostics_polish_execution/diagnostics_polish_execution.md`
- `docs/temporary/diagnostics_polish_execution/diagnostics_polish_closeout.md`

## Workstreams

### Workstream A: Documentation audit and contract cleanup

Deliverables:

- issue matrix covering conflicting or stale diagnostics guidance
- terminology cleanup proposal
- synchronized normative wording for host-audit boundary and inline repair accounting

Focus questions:

- where does `auditability` still imply the wrong behavior?
- which docs are normative versus historical?
- which examples currently encourage redundant rendering?

### Workstream B: Rendering policy redesign

Deliverables:

- a clear rendering policy for parse failure, single full failure, multi-unit failure, and partial success
- ownership table for headline / failed unit / `caused by:` / `help:`
- deduplication rules expressed in product terms rather than ad hoc code conditions

Focus questions:

- when does a block add distinct repair value?
- what should be collapsed versus expanded for LLM consumption?

### Workstream C: Runtime and presentation implementation

Deliverables:

- code changes removing structural redundancy
- code changes preserving useful evidence
- code review against the North Star rather than against local symmetry

Focus questions:

- can top-level failure details stop mirroring per-unit help?
- should outcome status be preserved more explicitly for presentation logic?
- where should deduplication happen: model shaping, presentation, or both?

### Workstream D: Tests and verification

Deliverables:

- regression tests for duplicate help suppression
- regression tests for `caused by:` information gain
- parity checks across MCP and CLI for the revised contract

Focus questions:

- which current tests only validate block presence?
- what assertions should directly encode LLM-facing DX?

## Proposed Wave Plan

### Wave 0: Inventory and issue matrix

- classify the relevant docs by authority and risk
- enumerate concrete redundancy bugs and conceptual conflicts
- record the current code paths that produce duplication

### Wave 1: Contract and terminology

- rewrite or tighten normative text around host audit versus inline repair accounting
- define information ownership and deduplication policy
- mark historical direction as historical where needed

### Wave 2: Rendering design

- define presentation strategies for distinct failure classes
- decide collapse rules for single full failure
- decide fallback rules for top-level versus per-unit `help:`

### Wave 3: Implementation

- update runtime shaping and presentation code
- keep edits minimal but principled
- avoid unrelated churn

### Wave 4: Test hardening

- add targeted assertions for redundancy suppression
- preserve partial-success accounting tests
- keep MCP and CLI parity visible

### Wave 5: Documentation sync and closeout

- sync public and internal docs
- mark temporary or historical documents appropriately
- record residual risks and future follow-up items

## Suggested Schedule

Estimated for one primary engineer supervising sub-agents:

- Day 1: Wave 0 plus initial Wave 1
- Day 2: finish Wave 1 and complete Wave 2
- Day 3: implementation
- Day 4: tests, doc sync, and closeout review
- Day 5: optional buffer for cleanup and broader doc sweep

## Sub-agent Coordination Rules

The project owner communicates only with the primary agent.

Rules for sub-agents:

- do not use popup communication
- do not contact the user directly
- do not broaden scope beyond this program without approval
- treat this document as the source of truth for the current effort
- preserve the host-audit boundary
- optimize for LLM-facing DX rather than for internal symmetry

## Acceptance Criteria

This program is complete only if all of the following are true:

- duplicate `help:` content is removed from the affected failure paths
- `caused by:` no longer restates the headline without adding value
- single full failure is measurably more compact than partial success
- partial success still preserves the accounting needed for safe repair
- docs consistently state that durable audit belongs to the host
- tests protect the new DX contract

## Immediate Next Actions

1. Build the documentation issue matrix.
2. Spawn parallel sub-agents for doc audit, code-path analysis, and test-plan analysis.
3. Integrate findings back into this program.
4. Implement the first wave of doc and code changes.
5. Verify behavior with targeted tests before final closeout.

## Phase-Two Follow-up

After the first-wave compact-failure and deduplication fixes land, continue with a second wave focused on broader confidence and long-tail drift prevention.

### Phase-Two Goals

- sweep historical or temporary docs that still risk teaching stale diagnostics behavior
- extend test coverage beyond `MATCH_INVALID_CONTEXT`
- exercise the CLI as a black-box surface using disposable sandbox files

### Phase-Two Workstreams

#### Workstream E: Historical-doc cleanup

Scope:

- `docs/temporary/*` files that still use overloaded `auditability` language
- temporary diagnostics plans that could be mistaken for active guidance
- downstream apply-splice temporary docs that inherit the old wording

Expected outcome:

- clearly marked historical documents
- reduced terminology drift
- less chance that future contributors reintroduce audit-shaped verbosity

#### Workstream F: Broader diagnostics coverage

Scope:

- non-context full-failure codes such as `UPDATE_TARGET_MISSING`, `DELETE_TARGET_MISSING`, and `TARGET_WRITE_ERROR`
- assertions that `caused by:` adds information instead of restating the headline
- parity checks where CLI and MCP should agree on compact failure rendering

Expected outcome:

- the DX contract is protected across more error classes
- redundancy suppression is not overfit to one mismatch family

#### Workstream G: CLI black-box testing

Primary sandbox:

- `temporary/spike1`

Testing philosophy:

- treat the CLI as the public executable contract
- derive cases from classic black-box methods, including:
  - equivalence classes
  - boundary values
  - error guessing
  - state-transition or workflow coverage where relevant
  - combinational coverage for small high-risk flag combinations

Expected outcome:

- a concise black-box matrix for high-value CLI failure and recovery paths
- concrete regressions or confidence signals grounded in executable behavior
