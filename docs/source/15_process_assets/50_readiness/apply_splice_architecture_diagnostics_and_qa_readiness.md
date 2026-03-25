# `apply_splice` Architecture, Diagnostics, And QA Readiness

## Readiness Scope

Close the implementation-entry gate for `apply_splice` architecture boundaries,
diagnostics-family readiness, source-of-truth precedence, and recursive QA structure.

## Target Gate

- `apply-splice-implementation-entry-ready`

## Required Inputs

- stable product boundary in {ref}`knowledge-interfaces-apply-splice-spec`
- source material: `docs/archive2026年3月24日/temporary/apply_splice_closure/architecture_diagnostics_test_draft.md`
- source material: `docs/archive2026年3月24日/temporary/apply_splice_technical_investigation.md`
- source material: `docs/archive2026年3月24日/temporary/apply_splice_closure/stage_summary.md`
- current patch/runtime presentation baselines used as the family reference point

## Open Risks

- shared helper extraction may drift above correctness substrate and start carrying
  splice semantics
- diagnostics-family names or blame hierarchy may remain under-specified for
  implementation
- QA may collapse into end-to-end demos unless layer ownership is explicit before coding

## Entry Conditions

- the narrow `apply_splice` product identity is already closed and not being reopened in
  architecture work
- implementation is still pre-coding or early enough that the architecture seam can be
  frozen before broad runtime work begins

## Exit Conditions

- the shared-vs-owned boundary is explicit and reviewable
- the minimum splice diagnostic family and blame hierarchy are locked tightly enough to
  code against
- source-of-truth precedence for pre-implementation and post-implementation drift is
  written down
- the recursive QA model and implementation-entry checklist are accepted

## Related Status Records

- future `30_records/60_status/` implementation-entry or architecture-lock status pages

## Shared Boundary

| Layer | May be shared | Must remain splice-owned |
| --- | --- | --- |
| Lower correctness substrate | path identity, connected-unit grouping, staged commit/rollback, affected-path summarization, generic path display, generic diagnostic rendering helpers, tiny numbered-excerpt codec | any logic that needs source-vs-target semantics, overlap legality, newline-fidelity policy, or splice vocabulary |
| Splice semantic layer | none beyond the shared helpers above | public tool identity, envelope grammar, parser, source/target selection resolver, same-file policy, source-byte transfer rules, splice-specific diagnostics vocabulary, and transfer/removal semantics |

## Minimal Diagnostic Family

| Code | Minimum meaning | Primary blame location |
| --- | --- | --- |
| `SPLICE_SOURCE_SELECTION_INVALID` | source numbered excerpt does not resolve truthfully | source selection block |
| `SPLICE_TARGET_SELECTION_INVALID` | target numbered excerpt does not resolve truthfully | target selection block |
| `SPLICE_SELECTION_TRUNCATED` | horizontal truncation or invalid omission form is present | offending selection line |
| `SPLICE_OVERLAP_ILLEGAL` | same-file source range overlaps the anchored target range under the v1 original-snapshot rule | action header |
| `SPLICE_TARGET_STATE_INVALID` | required target file or range does not exist for the action | target selection block or action header |
| `SPLICE_WRITE_ERROR` | commit-stage filesystem failure occurs after truthful planning | target path or destination anchor when no stronger authored span exists |
| `SPLICE_PARTIAL_UNIT_FAILURE` | some connected units committed while others failed | first failing unit plus committed/failed summary |

## Diagnostics Requirements

- failures begin with `error[CODE]: ...`
- blame hierarchy stays truthful: source mismatch to source block, target mismatch to
  target block, semantic ordering/overlap to action header, write-stage failure to the
  strongest available target-side location
- partial failure preserves committed `A/M/D`, failed groups, attempted `A/M/D`, and
  repair-oriented help
- target anchors are optional and evidence-based, never fabricated
- inline diagnostics remain the primary repair surface; any future persistence object must
  stay repair-oriented rather than becoming an audit-shaped sidecar contract

## Source-Of-Truth Policy

- before implementation ships: closed decision sources are the stable spec plus the
  accepted closure materials that carry implementation-facing detail
- after implementation ships: precedence is code plus passing tests, then stable spec,
  then tool docs/examples, then temporary notes and chat residue
- if docs and tests disagree, the same workstream must either update the stale docs or
  bring implementation/tests back to the intended behavior

## Recursive QA Model

Each top-level requirement should decompose outward through four layers:

1. shared substrate invariant
2. splice semantic invariant
3. user-visible presentation invariant
4. transport/parity invariant

The minimum scenario matrix must cover the full action basis plus same-file legality,
source-vs-target drift, target existence, newline fidelity, alias-path grouping,
partial-success accounting, evidence-based target anchors, and MCP/CLI parity.

## Implementation Entry Checklist

- [x] shared substrate extraction boundary is named and documented
- [x] numbered-excerpt codec contract exists and forbids horizontal truncation
- [x] splice-owned modules are named: parser, selection, runtime, presentation, server
      wiring
- [x] minimal splice diagnostic family is locked
- [x] blame hierarchy is locked for source mismatch, target mismatch, overlap/semantic
      error, and write-stage failure
- [x] success-path reporting is locked to family-compatible `A/M/D`
- [x] source-of-truth precedence is written down for pre- and post-implementation drift
- [x] recursive test decomposition and minimum scenario matrix are accepted
- [x] the first implementation batch can add tests at every required layer
- [x] grammar work is explicitly handed off to the canonical grammar artifact rather than
      being silently reopened inside architecture work
