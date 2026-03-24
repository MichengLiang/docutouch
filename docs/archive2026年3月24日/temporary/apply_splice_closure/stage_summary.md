# `apply_splice` Closure Stage Summary

## Status

An independent design-pass agent completed three review loops.
The main-agent review found the proposal strong enough to drive the next closure
wave, but not yet the final implementation artifact.
The closed product-boundary results have since been promoted into
`docs/apply_splice_spec.md`; this summary should now be read as a historical
closure register plus a pointer to the remaining implementation-facing gaps.

## Closed Decisions

- `apply_splice` remains a DocuTouch-owned tool with a public identity separate
  from `apply_patch`.
- The action basis now consists of eight transfer actions from the
  `Copy/Move × Append/Insert Before/Insert After/Replace` matrix plus one
  source-only removal primitive: `Delete Span`.
- Explicit omission phrases remain authoritative for splice selections:
  `... source lines omitted ...` and `... target lines omitted ...`.
- Splice selections use absolute numbered lines plus visible-content validation
  and forbid horizontal truncation.
- Same-file overlapping source/target ranges remain illegal for anchored
  transfer actions.
- Source bytes remain verbatim-preserved, including newline bytes.
- Program execution should follow connected-unit atomicity with partial success
  across disjoint units, rather than whole-program global atomicity.
- Cross-file move success summaries should align with the current implemented
  DocuTouch baseline and normalize to destination-side `M`.
- Closure review should treat code-plus-tests as the current runtime source of
  truth when examples in docs drift.

## Representation Gaps Still Being Closed

- Product-boundary questions are no longer open here; they already live in the
  stable spec.
- Canonical grammar text still needs to be written into a stable spec form.
- The selection validation relation needs to be promoted from prose to explicit
  formal semantics.
- Same-file multi-action semantics need their final spec wording, especially the
  ban on intermediate-state-dependent interpretation.
- Byte-span semantics need final spec wording that makes separator inclusion and
  EOF-without-newline behavior unambiguous.
- The minimum diagnostics code family still needs final locking.
- The minimum shared mutation substrate API boundary still needs final naming and
  extraction shape.

## Observed Drift Items

- The earlier move-summary drift in `docs/apply_patch_semantics_plan.md` has
  already been reconciled; keep code-plus-tests as the runtime source of truth
  if similar example drift reappears elsewhere.

## Next Recommended Step

Use the current closure results to write the next authoritative artifacts around
the remaining implementation-facing targets:

1. canonical splice grammar
2. formal selection and execution semantics
3. substrate and diagnostics closure map
