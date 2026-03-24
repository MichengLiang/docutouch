# `apply_splice` Integration Review

## Review Result

The two parallel drafts are materially aligned and can now serve as the current
implementation-prep baseline inside `docs/temporary/apply_splice_closure/`.
Stable product-boundary decisions have since been promoted into
`docs/apply_splice_spec.md`; this note should now be read as an integration check
for the remaining implementation-prep material.

Reviewed drafts:

- `formal_semantics_draft.md`
- `architecture_diagnostics_test_draft.md`

## What Now Looks Stable

- `apply_splice` remains a DocuTouch-owned tool surface distinct from
  `apply_patch`.
- The action basis now consists of eight transfer actions from
  `Copy/Move x Append/Insert Before/Insert After/Replace` plus one source-only
  removal primitive: `Delete Span`.
- Explicit omission phrases remain authoritative for splice selections.
- Selection semantics are now substantially formalized around numbered excerpts,
  double-lock validation, and contiguous denotation.
- Same-file execution now has a clear original-snapshot interpretation rule and
  rejects overlap in v1.
- Byte-span and newline preservation are now described as exact transfer of
  selected source bytes rather than reconstructed text.
- Program execution follows connected-unit atomicity and allows partial success
  across disjoint units.
- The shared-vs-owned architecture boundary is now concrete enough to guide
  extraction work.
- Diagnostics and QA now have a draft family contract and a recursive test
  decomposition.

## Remaining Integration Notes

- The move-summary baseline has been reconciled to the current implementation:
  move-shaped success normalizes to destination-side `M`.
- The stale move example in `docs/apply_patch_semantics_plan.md` has been fixed
  to match the runtime baseline.
- Before promoting these drafts into stable docs, the project should decide
  where the authoritative long-term home for each section belongs:
  - formal grammar / semantics
  - architecture boundary and shared substrate
  - diagnostics/test entry criteria

## Remaining Minor Gaps Before Promotion

- The canonical grammar draft should eventually be converted from a temporary
  draft into the final stable spec language and style.
- The diagnostics family may still need one final naming pass before code starts.
- The exact stable location of the source-of-truth policy should be decided so
  it does not remain only in temporary closure notes.

## Recommendation

The closure work has moved past broad conceptual uncertainty.
The next valuable step is no longer more discovery loops of the same kind.
The next valuable step is controlled promotion:

1. choose which parts of the closure drafts become stable docs
2. merge them into the long-term doc set
3. only then begin implementation against the promoted contract
