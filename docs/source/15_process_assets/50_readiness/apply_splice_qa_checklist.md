# `apply_splice` QA Checklist

## Readiness Scope

Provide the lightweight review checklist used to decide whether a proposed `apply_splice`
closure or implementation slice is ready to pass review without relying on ambient
memory.

## Target Gate

- `apply-splice-closure-review-pass`

## Required Inputs

- stable `apply_splice` boundary in {ref}`knowledge-interfaces-apply-splice-spec`
- `apply_splice_acceptance_criteria.md`
- `apply_splice_architecture_diagnostics_and_qa_readiness.md`
- the change, draft, or implementation slice being reviewed

## Open Risks

- reviewers may blur closed decisions with open questions
- proposals may accidentally collapse `apply_splice` into `apply_patch`
- diagnostics or QA claims may look plausible while still lacking evidence-backed closure

## Entry Conditions

- a concrete proposal, design slice, or implementation diff exists to review
- the reviewer has the current acceptance and architecture-readiness hosts available

## Exit Conditions

- the checklist sections below all have satisfactory answers
- any unresolved item is turned into an explicit blocker or follow-up, not silently
  ignored

## Related Status Records

- future `30_records/50_audit/` review findings for failed closure or QA gates

## Boundary

- does the proposal keep `apply_splice` separate from `apply_patch` as a public product
  identity?
- does it preserve the existing-span structural-operation boundary and keep inline
  authored text out of scope?
- does it treat the current action basis correctly: eight source/target transfer
  combinations plus the source-only `Delete Span` primitive?

## Contract Closure

- does it lock a canonical authored grammar?
- does it use explicit omission tokens rather than leaving bare `...` as an open option?
- does it define deterministic selection validation and same-file execution against one
  original snapshot?
- does it define program-level atomicity and partial-success behavior clearly enough to
  test?

## Architecture

- does it reuse only shared correctness substrate and avoid collapsing into vendored patch
  grammar?
- does it identify the minimum shared mutation boundary explicitly?
- does it avoid smuggling patch-private logic into splice-owned layers?

## Diagnostics

- does it align with the existing DocuTouch diagnostics philosophy?
- does it preserve truthful blame hierarchy?
- does it keep success-path `A/M/D` reporting aligned with the tool family?

## Discipline

- does it distinguish closed decisions from still-open closure points?
- does it avoid importing external-product heuristics that do not fit an internal,
  agent-native tool primitive?
- does it justify each major design component as necessary rather than merely plausible?
