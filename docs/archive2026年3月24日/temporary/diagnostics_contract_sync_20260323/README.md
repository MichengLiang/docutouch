# Diagnostics Contract Sync 2026-03-23

## Status

- Active temporary coordination folder for the next `apply_patch` diagnostics wave.
- Created after the project owner accepted the new partial-failure layout, full committed/failed enumeration, and failed patch source persistence direction.
- This folder is the handoff surface for the upcoming 5.4 worker execution wave.

## Purpose

This folder exists to lock one thing before code work expands again:

- the diagnostics contract has changed in substance, not only in wording

The key accepted changes are:

1. partial failure must enumerate all committed and failed paths that matter for repair
2. omission prose such as `... and N more committed changes` is no longer acceptable in the public repair contract
3. failed patch source persistence is allowed when the patch did not already come from a stable file path
4. audit-shaped sidecars, replay caches, and secondary JSON payloads remain rejected
5. multiline cause blocks and hierarchy alignment now matter as first-class DX work, not aesthetic polish

## Authority Order

When documents disagree, use this precedence order for the current wave:

1. this folder
2. `docs/apply_patch_diagnostics_spec.md`
3. `docs/diagnostics_polish_spec.md`
4. `docutouch-server/tool_docs/apply_patch.md`
5. historical temporary docs only where they do not conflict with the accepted 2026-03-23 direction

## Deliverables For The Wave

- synchronized live docs
- runtime support for failed patch source persistence
- renderer support for the accepted partial-failure hierarchy
- full committed / failed enumeration in public diagnostics
- regression and black-box coverage for the new contract

## Worker Constraint

Sub-agents / workers may not call `popup_ask_user`.
Only the primary agent communicates with the user through popup.

## Files In This Folder

- `execution_plan.md`
  stage plan, priorities, worker split, and acceptance criteria
- `doc_sync_matrix.md`
  what was already synchronized and what still needs a downstream sweep
