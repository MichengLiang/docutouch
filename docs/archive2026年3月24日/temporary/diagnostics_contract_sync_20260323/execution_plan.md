# Diagnostics Contract Sync Execution Plan

## Goal

Turn the accepted diagnostics direction into one executable wave without reopening the contract during implementation.

The wave is complete only when:

1. live docs teach the accepted contract
2. runtime behavior matches the accepted contract
3. CLI and MCP expose the same visible diagnostics shape
4. tests protect the new repair-facing behavior

## Locked Decisions

The following are not open for debate in this wave:

1. partial failure must show all committed and failed repair-relevant paths
2. omission prose for committed-path accounting is out
3. failed patch source persistence is in when the patch was inline / stdin / MCP text and failed
4. audit-shaped sidecars, replay caches, and secondary JSON payloads remain out
5. multiline cause blocks must render as real sub-blocks, not raw newline spill
6. hierarchy must separate transaction-level accounting from per-group failure detail

## Priority Order

### Stage 0: Doc Sync Lock

Purpose:

- ensure future workers are not guided by stale documentation

Scope:

- finish live-document synchronization
- mark stale temporary records as partially superseded where needed
- record remaining downstream doc sweep items explicitly

Acceptance:

- active `apply_patch` docs no longer teach strict inline-only repair
- active `apply_patch` docs no longer justify committed-path compression
- failed patch source persistence is described as a repair object, not as an audit layer

### Stage 1: Runtime Data Plumbing

Purpose:

- make failed patch source persistence real in the lower layers

Scope:

- define the hidden workspace directory contract for persisted failed patch sources
- keep original patch file paths when a stable patch file already exists
- persist inline / stdin / MCP patch text on failure only
- thread the resulting patch path through runtime diagnostics structures

Acceptance:

- failure diagnostics can point to a real patch path instead of bare `<patch>` when persistence is required
- no secondary JSON report or audit cache is introduced
- success path remains free of new artifacts

### Stage 2: Presentation Rewrite

Purpose:

- make the visible diagnostics shape match the accepted layout

Scope:

- remove committed-list compression and omission prose
- separate transaction summary from per-group failure blocks
- render multiline failed-group `caused by` output as an indented sub-block
- add per-group patch pointers when source information exists
- preserve compact single full failure where the accepted contract still wants compactness

Acceptance:

- no `... and N more committed changes` remains in the public patch diagnostics contract
- failed-group multiline causes stay inside their block
- accepted ASCII layout is recognizable in CLI and MCP outputs

### Stage 3: Tests And Black-Box Verification

Purpose:

- convert the new contract into durable regression protection

Scope:

- renderer tests for multiline indentation, full enumeration, patch-path rendering, and block hierarchy
- runtime tests for failed patch source persistence behavior
- CLI/MCP parity tests for the new partial-failure shape
- black-box cases covering empty patch, mismatch, multi-failure partial success, and persisted patch-source references

Acceptance:

- tests fail without the new behavior and pass with it
- CLI and MCP stay aligned on the accepted shape
- the tmp black-box matrix covers at least one multi-failure partial-success case

### Stage 4: Downstream Doc Sweep And Closeout

Purpose:

- prevent stale downstream design docs from reintroducing the old direction

Scope:

- update or mark downstream `apply_splice` and closure docs that still teach strict inline-only wording where it now conflicts with platform direction
- record residual risks and any intentionally deferred polish

Acceptance:

- no active or near-active doc teaches the rejected inline-only / no-persisted-patch-source position for patch diagnostics
- closeout names any remaining downstream historical docs that are intentionally left as historical only

## Recommended Worker Split

### Worker A: Persistence Plumbing

Ownership:

- `docutouch-core/src/patch_runtime.rs`
- relevant lower-layer patch path plumbing
- any persistence helper location chosen for the wave

Deliver:

- failed patch source persistence and path threading

### Worker B: Presentation Layer

Ownership:

- `docutouch-core/src/patch_presentation.rs`

Deliver:

- accepted partial-failure layout
- full enumeration
- multiline cause indentation

### Worker C: Tests / Verification

Ownership:

- `docutouch-core` tests
- `docutouch-server/tests/stdio_smoke.rs`
- `docutouch-server/tests/cli_smoke.rs`

Deliver:

- contract assertions and black-box parity coverage

### Worker D: Downstream Doc Sweep

Ownership:

- remaining downstream doc set listed in `doc_sync_matrix.md`

Deliver:

- consistency notes, status flips, and wording repairs

## Worker Rules

1. Workers must not call `popup_ask_user`.
2. Workers are not alone in the codebase and must not revert other edits.
3. Workers should preserve the accepted contract; they are not to renegotiate it.
4. If a worker finds a contract conflict, it should report it rather than silently choosing the older doc.

## Review Checklist

Before code lands, verify:

- persisted patch path appears only when needed
- no audit/report sidecar system has been reintroduced
- every committed and failed repair-relevant path is visible in partial failure
- group blocks remain structurally aligned even when causes are multiline
- docs, runtime, and tests all teach the same contract
