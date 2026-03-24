# `apply_patch` Diagnostics Implementation Plan

## Status

- Historical plan retained for reference
- Superseded first by the inline-only diagnostics direction and then in part by the
  2026-03-23 direction that restores failed patch source persistence while still
  rejecting audit-shaped tool-layer sidecars
- Do not use this file as the active execution plan for new diagnostics DX work
- See `docs/temporary/diagnostics_dx_repair_program.md` for the current repair
  program

## Phase 0. Design Lock

Status: completed

Tasks:

- [x] Re-read current semantics, UX, roadmap, and hardening docs
- [x] Confirm that source-span grade execution diagnostics are the next highest-value target
- [x] Separate design specification from implementation schedule

## Phase 1. Diagnostic Data-Flow Audit

Status: completed

Tasks:

- [x] Trace how source-map information currently flows from parser to runtime to server
- [x] Identify which execution failure classes already carry action/hunk metadata
- [x] Identify where source-line/source-column information is lost
- [x] Record which failure classes can gain truthful spans with low risk

Deliverable:

- implementation notes mapping failure classes to available source evidence

## Phase 2. Source-Span Grade Execution Diagnostics

Status: completed

Tasks:

- [x] Extend lower layers so more execution failures retain exact patch-source locations
- [x] Surface `source_line` / `source_column` when the mapping is robust
- [x] Preserve truthful fallback behavior when the mapping is not robust
- [x] Add tests for representative execution failure classes

Deliverable:

- a tighter failure surface for execution-stage diagnostics

## Phase 3. Optional Target-Side Anchoring

Status: completed for the currently selected mismatch scope

Tasks:

- [x] Identify one or two high-value mismatch classes where a secondary target anchor materially helps
- [x] Add a compact target anchor without turning diagnostics into long excerpts
- [x] Validate that the extra anchor reduces ambiguity rather than adding noise

Deliverable:

- one-primary-span plus optional-secondary-anchor behavior in selected cases

## Phase 4. Warning / Error Taxonomy Tightening

Status: completed for the current warning/error set

Tasks:

- [x] Review current warning codes and error codes for naming consistency
- [x] Group future classes by stage and cause where useful
- [x] Ensure rendered tone stays consistent across success warnings and failure errors

Deliverable:

- a more systematic diagnostics grammar

## Phase 5. Recovery Contract Upgrade

Status: historically completed, but now superseded by the decision to remove
tool-managed failure artifacts and rely on host-level tool-call audit logs

Tasks:

- [x] Review the historical `failed-groups.json` shape
- [x] Identify which fields were previously considered prose-heavy from a repair-loop perspective
- [x] Record that this line of work is now superseded by inline-only diagnostics plus host audit logs

Deliverable:

- a recorded historical branch that should no longer guide current product work

## Phase 6. Validation and Review

Status: completed

Tasks:

- [x] Run targeted tests for source-span-rich and source-span-poor cases
- [x] Verify that no fake precision was introduced
- [x] Check that warning-free success remains unchanged
- [x] Review whether partial-failure repair accounting stayed intact

Deliverable:

- implementation review summary and ship-readiness assessment

## Review Standard

The implementation should be judged against one sentence:

- rustc-like honesty, not rustc cosplay

## Progress Log

- 2026-03-20: detailed diagnostics specification created
- 2026-03-20: execution plan separated from the design specification
- 2026-03-20: parser source maps now retain `Move to` locations so commit-stage destination failures can point back to truthful patch lines
- 2026-03-20: execution diagnostics now preserve commit-stage source spans and selected context-mismatch target anchors end-to-end through server rendering
- 2026-03-22: team direction changed; tool-managed failure sidecars are to be retired in favor of self-contained inline diagnostics and host-level audit logs
