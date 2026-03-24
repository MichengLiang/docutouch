# `apply_splice` Closure Workspace

## Purpose

This directory records the pre-implementation closure work for `apply_splice`.

The goal is not to rush into code.
The goal is to close the remaining semantic, architectural, and diagnostics gaps
until the design is formal enough to drive parser, runtime, and test work
without avoidable drift.

Product-boundary decisions that have already been promoted into
`docs/apply_splice_spec.md` are not reopened in this directory.
This workspace now exists to close remaining implementation-facing semantics,
diagnostics, QA, and extraction details.
When diagnostics wording in this closure workspace conflicts with the live
Rust-docs contract, prefer `docs/temporary/diagnostics_contract_sync_20260323/`,
`docs/apply_patch_diagnostics_spec.md`, `docs/diagnostics_polish_spec.md`, and
`docs/apply_splice_spec.md`.

## Working Method

This closure pass follows a review-loop workflow:

1. independent proposal generation
2. main-agent review and QA
3. targeted revision requests
4. repeated closure passes until the remaining open points are explicit,
   justified, and bounded

## Expected Artifacts

- proposal drafts from the independent design pass
- review notes and QA findings
- narrowed decisions
- explicit remaining open questions, if any

## Scope

The closure work focuses on:

- goal decomposition and obstacle analysis
- canonical grammar and authored surface
- selection semantics and validation rules
- same-file execution semantics
- byte fidelity and newline handling
- program-level atomicity and partial-success semantics
- diagnostics obligations
- shared mutation substrate boundaries
