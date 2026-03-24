# (process-assets-apply-splice-engineering-hardening-plan)=
# Apply Splice Engineering Hardening Plan

## Purpose

Host the post-v1 engineering-quality wave for `apply_splice`, focusing on shared-substrate
extraction, duplication reduction, presentation hardening, and doctrine propagation rather than
new end-user-facing capability.

## Source Basis

- {ref}`knowledge-architecture-apply-splice-architecture`
- {ref}`knowledge-operations-upstream-sync-and-compatibility`
- {ref}`knowledge-interfaces-apply-splice-spec`
- {ref}`process-assets-apply-splice-implementation-plan`
- `30_records/60_status/apply_splice_implementation_status.md`
- current engineering-quality investigation covering substrate reuse, transport duplication,
  and presentation/test coverage gaps

## Target Outcome

- the repo's first-entry process assets and follow-up work treat vendored
  `codex-apply-patch` as internal substrate rather than an upstream architecture leash
- `apply_splice` owns its semantic layers while reusing more genuine substrate-level helpers
- splice selection semantics have one clearer implementation authority instead of duplicated
  text-vs-byte resolution logic
- patch/splice transport and presentation shells have materially lower drift risk
- QA and documentation closure can truthfully claim a lower-maintenance architecture state

## Scope

- doctrine propagation into new process assets for this hardening wave
- splice selection-authority reduction
- low-risk shared-substrate extraction where the logic is genuinely splice-agnostic
- transport-shell and presentation-helper duplication reduction where the boundary is generic
- test and documentation closure for the changed seams

## Non-Goals

- collapsing `apply_patch` and `apply_splice` into one tool identity
- changing the accepted `apply_splice` action basis or broadening its semantic scope
- reopening already-accepted product-boundary decisions
- performing a generic upstream resync of the vendored fork
- style-only refactors that do not materially lower drift risk

## Milestones And Duration

| Milestone | Target Outcome | Expected Duration | Entry Condition | Exit Condition |
| --- | --- | --- | --- | --- |
| M1 | Engineering hardening wave has canonical plan and gate hosts | short wave slice | investigation findings are stable enough to drive work | workers can execute against one clear authority instead of chat residue |
| M2 | Splice selection semantics have one clearer implementation authority | medium wave slice | M1 host exists and current duplication seam is understood | text-level and byte-level selection behavior no longer require parallel contract edits |
| M3 | Low-risk shared substrate extraction reduces duplicate path/state helpers | medium wave slice | M1 host exists and semantic-vs-substrate seams are explicit | affected-path, path-resolution, or equivalent substrate helpers have fewer duplicate owners |
| M4 | Transport/presentation shell drift risk is reduced | medium wave slice | M1 exists and current server/presentation duplication is mapped | patch/splice shell layers share more generic helper structure without erasing splice-owned wording |
| M5 | QA, docs, and status closure match the post-wave state | end-of-wave integration pass | M2-M4 converge | tests, docs, and records can describe the hardening wave truthfully |

## Dependency Strategy

- keep the accepted product boundary fixed while hardening the engineering substrate
- treat shared extraction as valid only when the helper remains splice-agnostic
- let doctrine/host creation happen first so implementation slices do not improvise authority
- land QA and docs closure after the ownership seams settle rather than in parallel guesswork

## Parallelization Plan

- one worker may own process-asset host creation and status wiring
- one worker may own splice selection-authority cleanup
- one worker may own server/presentation-shell duplication reduction
- one worker may own low-risk substrate extraction, but only after checking that the moved logic
  is genuinely semantic-free
- integration review and QA closure remain main-agent owned after the implementation slices land

## Acceptance Strategy

- the wave counts as complete only if duplication is reduced at the actual authority seam, not
  merely moved into a new helper layer with the same number of owners
- extracted helpers must stay substrate-level and must not smuggle splice-owned semantics into
  patch-owned or generic layers
- patch and splice tests must continue to prove their accepted public contracts after the
  hardening wave
- docs and records must stop overstating reuse or understating remaining seams

## Risk And Replan Triggers

- if a proposed extraction requires moving splice-specific diagnostics, same-file policy, or
  newline-fidelity rules into a generic layer, stop and narrow the extraction
- if transport deduplication starts to change user-visible output wording, stop and separate the
  contract change from the quality wave
- if low-risk substrate extraction proves to require broad patch-runtime surgery, split the work
  into a later wave instead of hiding it inside this plan
- if docs closure reveals conflicting accepted doctrine, do not treat prose edits as sufficient;
  reopen the implementation slice first

## Related Work Packages

- future worker-owned packages for selection-authority cleanup
- future worker-owned packages for substrate extraction
- future worker-owned packages for transport/presentation shell hardening
- future documentation/status closure package for the end-of-wave pass

## Related Records

- future `30_records/60_status/` closeout page for this hardening wave
- future `30_records/50_audit/` findings if an extraction or QA pass fails review
