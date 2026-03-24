# `apply_splice` Implementation Plan

## Status

- Scope: detailed implementation plan for the `apply_splice` tool
- Complements `docs/apply_splice_spec.md`
- Assumes product-boundary questions already live in the stable spec; this plan
  is for implementation sequencing, not for reopening tool identity

## Phase 0. Design Lock

Status: completed

Tasks:

- [x] Confirm the tool identity is narrow and distinct from `apply_patch`
- [x] Confirm the source/target two-axis action model
- [x] Confirm that selections use absolute line numbers plus visible-content validation
- [x] Confirm that horizontal truncation is forbidden inside splice selections
- [x] Confirm that same-file source/target resolution uses the original snapshot
- [x] Confirm that overlapping same-file source/target ranges are illegal
- [x] Confirm narrow target-existence rules and verbatim newline preservation

## Phase 1. Grammar and Parsing Strategy

Status: pending

Tasks:

- [ ] Decide the exact envelope and action header spellings
- [ ] Decide how source and target selection blocks are represented in the parser
- [ ] Decide whether any apply_patch parsing utilities can be safely reused
- [ ] Lock the canonical grammar before runtime work begins

Deliverable:

- a stable authored grammar for `apply_splice`

## Phase 2. Selection Resolution Engine

Status: pending

Tasks:

- [ ] Resolve contiguous source selections from numbered excerpt blocks
- [ ] Resolve contiguous target selections from numbered excerpt blocks
- [ ] Validate line-number and content agreement
- [ ] Reject ambiguous or horizontally truncated selections

Deliverable:

- a deterministic source/target selector

## Phase 3. Atomic Transfer Runtime

Status: pending

Tasks:

- [ ] Implement copy semantics
- [ ] Implement move semantics
- [ ] Implement delete-span semantics
- [ ] Implement append / insert before / insert after / replace target modes
- [ ] Reuse or mirror file-group atomicity guarantees for connected source/destination updates

Deliverable:

- a safe splice runtime with explicit source and target effects

## Phase 4. Diagnostics and Tests

Status: pending

Tasks:

- [ ] Add stable diagnostics for drift and mismatch cases
- [ ] Add tests for source drift, target drift, overlap edge cases, and move atomicity
- [ ] Keep diagnostics aligned with the existing DocuTouch repair-oriented style

Deliverable:

- test-backed diagnostics and failure surface

## Phase 5. Prompt-Facing Guidance and Review

Status: pending

Tasks:

- [ ] Document the narrow capability boundary clearly
- [ ] Explain the difference from `apply_patch`
- [ ] Review whether the action surface still feels natural and unsurprising
- [ ] Prepare acceptance summary

Deliverable:

- user-facing contract and review package

## Review Standard

- reference-preserving, not text-generating
- narrow and comfortable, not DSL-heavy
- atomic and fully accounted-for move operations

## Progress Log

- 2026-03-20: detailed `apply_splice` design specification created
- 2026-03-20: implementation schedule separated from the design specification
- 2026-03-20: design-lock decisions added for vendored-boundary clarification, same-file overlap policy, target existence, newline fidelity, and diagnostics blame order
