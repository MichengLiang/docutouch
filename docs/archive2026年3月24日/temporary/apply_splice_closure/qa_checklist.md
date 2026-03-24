# `apply_splice` QA Checklist

## Boundary

- Does the proposal keep `apply_splice` separate from `apply_patch` as a public
  product identity?
- Does it preserve the existing-span structural-operation boundary and reject
  inline authored text?
- Does it treat the current action basis correctly: eight source/target transfer
  combinations plus the source-only `Delete Span` primitive, rather than a
  product-scope menu?

## Contract Closure

- Does it lock a canonical authored grammar?
- Does it treat explicit omission tokens as authoritative rather than leaving
  them as an open alternative to bare `...`?
- Does it define a deterministic selection validation relation?
- Does it define same-file execution semantics against one original snapshot?
- Does it define program-level atomicity and partial-success behavior?

## Architecture

- Does it reuse shared correctness substrate without collapsing into vendored
  patch grammar?
- Does it identify the minimum shared mutation substrate boundary?
- Does it avoid smuggling patch-private logic into splice-specific layers?

## Diagnostics

- Does it align with the existing DocuTouch diagnostics philosophy?
- Does it preserve truthful blame hierarchy?
- Does it keep success-path `A/M/D`-style reporting aligned with the existing
  tool family?

## Discipline

- Does it distinguish closed decisions from still-open closure points?
- Does it avoid external-product heuristics that do not fit an internal
  agent-native tool primitive?
- Does it justify each major design component as necessary rather than merely
  plausible?
