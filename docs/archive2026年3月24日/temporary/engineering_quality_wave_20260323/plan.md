# Engineering Quality Wave Plan

## Status

- Drafted on 2026-03-23 after the diagnostics contract sync wave reached green tests.
- This document is for the next wave only: engineering quality, ownership cleanup, and maintainability hardening.
- It is intentionally downstream of the accepted diagnostics contract; it does not reopen product-facing DX decisions already locked in the previous wave.

## Purpose

The previous wave already landed the user-visible contract changes:

- failed patch source persistence
- full committed / failed enumeration for partial failure
- cleaner partial-failure hierarchy
- multiline `caused by` block rendering
- updated tool docs and examples

What remains is no longer primarily product behavior.
What remains is engineering quality:

- semantic authority drift
- duplicated rules across layers
- test harness fragility
- mixed ownership of execution anchors vs display anchors

This wave exists to reduce the cost and risk of future diagnostics work.

## Non-Goals

This wave should **not**:

1. redesign the accepted diagnostics contract again
2. reopen whether partial failure should enumerate all paths
3. remove failed patch source persistence
4. add new end-user-facing tools
5. chase style-only refactors with no maintenance payoff

## Guiding Principle

The target is:

> one rule, one owner, one place

If a rule currently has to be remembered in multiple adapters or layers, that is a refactor target.

## Problem Statement

The current system is correct and tested, but the implementation has accumulated several quality risks:

### Q1. CLI and MCP do not truly share one semantic authority

`docutouch-server` still presents itself as if `ToolService` were the shared semantic authority for every transport, but the CLI path executes several commands directly against `docutouch-core`.

Current consequence:

- parity is protected mainly by tests and discipline
- new semantics, such as patch-source provenance, must be wired into CLI separately
- future changes risk adapter drift

### Q2. Patch-source provenance rules are split across too many layers

At the moment, patch-source ownership is partially spread across:

- CLI adapter
- runtime
- presentation

Current consequence:

- the same conceptual rule can be re-derived in more than one place
- precedence bugs are easier to introduce
- future changes require wide awareness

### Q3. Test harness process lifecycle is still fragile

`cli_smoke` now has timeout / cleanup guard work, but process spawning and cleanup discipline are still not centralized across the full diagnostics test surface.

Current consequence:

- a future hang can again leave hot orphan processes behind
- child lifecycle policy is not yet one reusable abstraction
- `stdio_smoke` still leans on more ad hoc patterns than ideal

### Q4. Execution anchoring and display anchoring are still too coupled

`PatchPresentationContext` and related service-layer decisions currently mix:

- where semantics should execute
- how paths should be rendered

Current consequence:

- transport logic and display logic remain too entangled
- renderer evolution risks dragging service-layer concerns with it

## Wave Goal

By the end of this wave, the project should be able to say:

1. CLI and MCP share a clearer semantic adapter boundary
2. patch-source provenance has one primary owner
3. child-process lifecycle in smoke tests is guarded by reusable helpers
4. execution anchor vs display anchor responsibilities are more explicit
5. future diagnostics changes can land with fewer cross-file edits and fewer parity hazards

## Workstreams

## Workstream A: Shared Patch Invocation Adapter

### Goal

Stop making patch semantics depend on separate CLI-vs-server wiring decisions.

### Scope

- introduce or extract one shared patch invocation path above raw `docutouch-core` calls
- encode patch-source provenance as explicit input to that shared path
- keep CLI-specific input collection separate from semantic execution
- keep MCP/tool-service argument handling separate from semantic execution

### Preferred Ownership

- `docutouch-server/src/cli.rs`
- `docutouch-server/src/tool_service.rs`
- possibly one new small shared helper module in `docutouch-server/src/`

### Desired Outcome

- CLI no longer has to privately remember semantic patch behavior
- MCP/CLI differences become transport-shape differences, not semantic duplication

### Acceptance

1. patch execution semantics are triggered through one shared adapter API
2. patch-source provenance is explicit in that API
3. CLI/MCP parity for patch behavior remains green
4. the adapter boundary is smaller and easier to explain than today

## Workstream B: Patch-Source Provenance Ownership Cleanup

### Goal

Give patch-source provenance one obvious owner.

### Scope

- audit every place that currently re-derives patch source paths
- decide where `*** Patch File:` interpretation truly belongs
- decide whether presentation should still parse patch metadata or consume only runtime-shaped state
- reduce duplicated precedence logic

### Preferred Ownership

- `docutouch-core/src/patch_runtime.rs`
- `docutouch-core/src/patch_presentation.rs`
- shared exports in `docutouch-core/src/lib.rs`

### Desired Outcome

- provenance precedence is written once
- presentation consumes already-shaped information rather than replaying policy
- unreadable/stale patch-file references cannot bypass the intended repair-object guarantee

### Acceptance

1. one primary layer owns provenance precedence
2. no duplicated `*** Patch File:` semantics remain without explicit justification
3. tests cover inline patch, patch-file hint, embedded patch-file, and fallback persistence

## Workstream C: Smoke-Test Process Harness Hardening

### Goal

Make smoke tests safe even when regressions or interrupted runs happen.

### Scope

- centralize child-process spawn / wait / timeout / kill behavior
- centralize stdio server call timeout behavior where practical
- avoid leaving hot `cargo` / `cli_smoke` / child chains after failure
- keep helpers Windows-compatible and pragmatic

### Preferred Ownership

- `docutouch-server/tests/cli_smoke.rs`
- `docutouch-server/tests/stdio_smoke.rs`
- possibly one shared `tests/common` helper if that reduces duplication materially

### Desired Outcome

- no important smoke path relies on bare `wait_with_output()` with no timeout guard
- child cleanup is consistent
- future hangs fail fast and noisily rather than cooking the machine

### Acceptance

1. the main spawned-process helpers have timeout + cleanup policy
2. interrupted or timed-out test paths do not leave obvious hot child chains
3. the helper behavior itself is covered by at least one regression-style test or focused validation path

## Workstream D: Anchor Context Decoupling

### Goal

Separate execution anchoring from display anchoring where it improves clarity.

### Scope

- audit `PatchPresentationContext`
- decide whether one object should still carry both execution and display concerns
- split only if the split produces clearer ownership and fewer adapter leaks

### Preferred Ownership

- `docutouch-server/src/tool_service.rs`
- `docutouch-core/src/patch_presentation.rs`
- any adjacent shared types that currently carry mixed responsibility

### Desired Outcome

- service layer decides execution anchor
- presentation layer decides rendering from already-owned display inputs
- the boundary becomes easier to reason about in future changes

### Acceptance

1. context objects have clearer single-purpose ownership
2. no transport layer has to know renderer internals just to create a context
3. tests still pass unchanged or with cleaner setup

## Workstream E: Post-Refactor Contract Re-Verification

### Goal

Ensure quality refactors did not silently change the accepted diagnostics contract.

### Scope

- rerun contract-critical test suites
- rerun the example showcase script
- manually spot-check at least one compact full failure, one partial failure, and one persisted patch-source case

### Acceptance

1. `cargo test -p docutouch-core`
2. `cargo test -p docutouch-server`
3. `uv run python example/1.py`
4. no regression in committed enumeration, patch-path rendering, or failure hierarchy

## Workstream F: Documentation Closure

### Goal

Make the maintainer-facing source of truth match the codebase that ships out of this wave.

### Scope

- update stable README / maintainer-facing docs that currently misstate transport inventory or semantic authority
- remove stale copy-paste paths from runnable examples where practical
- ensure shipped `search_text` status is described as shipped, not still hypothetical
- record the outcome of this wave strongly enough that future maintainers do not need to reconstruct it from temporary chat history

### Preferred Ownership

- `README.md`
- `docs/README.md`
- `docs/roadmap.md`
- `docutouch-server/README.md`
- `docutouch-server/HTTP_GUIDE.md`

### Desired Outcome

- stable docs describe the actual transport surface
- docs no longer overclaim that all transport semantics already live in one place when that is still work in progress
- runnable examples do not fail because of stale flags or legacy checkout paths

### Acceptance

1. root/server docs agree on stdio MCP, CLI adapter, and fixed-workspace HTTP transport inventory
2. `search_text` is described as shipped, with future work framed as optimization rather than initial introduction
3. HTTP startup examples include a valid auth story (`--api-key`, `--api-key-env`, or `--disable-auth`)
4. Workstream E includes one doc audit pass alongside code/test re-verification

## Recommended Parallel Split

The max concurrency is 6, but I would not start with 6.
Recommended first pass: **4 workers**.

### Worker 1: Shared Patch Adapter

Ownership:

- `docutouch-server/src/cli.rs`
- `docutouch-server/src/tool_service.rs`
- optional new shared adapter module in `docutouch-server/src/`

Mission:

- centralize patch invocation semantics

### Worker 2: Provenance Ownership

Ownership:

- `docutouch-core/src/patch_runtime.rs`
- `docutouch-core/src/patch_presentation.rs`

Mission:

- collapse duplicated patch-source precedence logic

### Worker 3: Harness Hardening

Ownership:

- `docutouch-server/tests/cli_smoke.rs`
- `docutouch-server/tests/stdio_smoke.rs`
- optional shared test helper module

Mission:

- timeout / cleanup / child lifecycle guard

### Worker 4: Review + Re-Verification

Ownership:

- read-only review first
- then narrow follow-up edits in uncovered gaps

Mission:

- validate that refactors did not distort the accepted contract

## Optional Fifth Worker

Only spawn a fifth worker if the first four uncover enough cleanly separable work.

Best candidate:

- context/anchor type cleanup once the shared patch adapter direction is known

## Optional Sixth Worker

If docs drift is already confirmed during review, spawn one docs-only worker rather than leaving doc sync as implicit cleanup.

Best candidate:

- stable README / roadmap / guide alignment after the code ownership changes settle

## Worker Rules

1. Workers may not call `popup_ask_user`.
2. Workers are not alone in the codebase and must not revert one another.
3. Workers should treat the accepted diagnostics contract as locked.
4. Workers should surface architectural disagreement explicitly rather than silently choosing one path.
5. If a worker has to touch more than its planned slice, it should stop and report.

## Review Standard

This wave should be reviewed against three questions:

1. Did we reduce semantic duplication, or only move it around?
2. Did we reduce future drift risk, or only preserve current tests?
3. Did we keep the accepted user-facing diagnostics contract intact?

If the answer to any of these is “no,” the refactor is incomplete.

## Suggested Execution Order

1. Worker 1 and Worker 2 start first, because ownership boundaries drive everything else.
2. Worker 3 starts in parallel, because harness hardening is mostly disjoint.
3. Worker 4 begins as a read-only reviewer and turns into a fix worker only if real issues appear.
4. After those slices converge, do one focused integration pass, rerun the contract suites, and do one doc audit pass against the changed architecture files.

## Ship Criteria

This wave should count as complete only if all of the following are true:

1. CLI/MCP patch semantics are easier to trace to one shared authority
2. patch-source provenance precedence no longer lives in multiple layers without a clear owner
3. smoke-test child lifecycle is guarded enough that regressions do not easily strand hot orphan chains
4. `cargo test -p docutouch-core` passes
5. `cargo test -p docutouch-server` passes
6. `uv run python example/1.py` still demonstrates the intended diagnostics surface
7. stable maintainer-facing docs no longer misdescribe the shipped transport surface or current `search_text` status

## Why This Wave Is Worth Doing

The previous wave already made the product better.
This wave makes the codebase cheaper to keep correct.

That matters because the diagnostics system is now good enough that future mistakes are more likely to come from:

- ownership confusion
- duplicated semantic rules
- adapter drift
- fragile tests

This plan is therefore not “nice to have cleanup.”
It is the work required to stop the next diagnostics wave from becoming expensive and error-prone.
