# `apply_splice` Acceptance And QA Criteria

## Readiness Scope

Define the release-entry readiness surface for the first accepted implementation closure
of `apply_splice`, covering grammar, semantics, transfer fidelity and newline-boundary normalization, atomicity, diagnostics,
transport parity, and documentation truthfulness.

## Target Gate

- `apply-splice-v1-release-ready`

## Required Inputs

- stable contract source in {ref}`knowledge-interfaces-apply-splice-spec`
- implementation-facing closure inputs from the original acceptance/closure drafts
- parser, selection, runtime, presentation, and transport test evidence
- tool docs and stable docs that match the implemented behavior

## Open Risks

- code, tests, and docs may drift on exact grammar or diagnostics wording
- same-file behavior, rollback, and partial-success accounting may appear correct only in
  outer demos unless lower-layer evidence exists
- examples may lag behind the actual runtime and misstate the user-visible contract

## Entry Conditions

- the stable `apply_splice` boundary is already closed enough that v1 implementation can
  be judged against pass/fail gates rather than open product discovery
- the closure artifacts needed for implementation-facing detail have been reconciled into
  a reviewed readiness surface

## Exit Conditions

- all top-level acceptance goals below are satisfied
- every criteria family has explicit pass evidence at the required layers
- the negative suite and permanent regression obligations exist and pass
- stable docs, tool docs, tests, and examples describe the same observable contract

## Related Status Records

- future `30_records/60_status/` pages for `apply_splice` release readiness and closeout

## Evidence Classes

| Evidence | Meaning |
| --- | --- |
| `E1` | spec or readiness text updated to match implemented behavior |
| `E2` | lower-layer parser, selection, runtime, or shared-substrate tests |
| `E3` | presentation evidence for rendered success/failure and blame behavior |
| `E4` | MCP/CLI transport parity evidence |
| `E5` | durable regression coverage kept in the permanent suite |

## Top-Level Acceptance Goals

- `AG-01 Narrow Tool Identity`: `apply_splice` remains a narrow structural-operation
  tool over existing spans and still forbids inline authored text.
- `AG-02 Deterministic Selection Semantics`: selections resolve by absolute line numbers
  plus visible-content matching, with contiguous denotation and no horizontal
  truncation.
- `AG-03 Deterministic Same-File Behavior`: same-file programs resolve against one
  original snapshot and reject overlap or intermediate-state dependence in v1.
- `AG-04 Transfer Fidelity And Line-Boundary Preservation`: ordinary transfer keeps source content and newline bytes intact where possible, while EOF-final-line selections that would otherwise concatenate target lines are normalized in the result without rewriting source state.
- `AG-05 Atomic And Accounted Mutation`: connected units commit atomically, disjoint
  units may partially succeed, and the output keeps the result repair-accounted.
- `AG-06 Honest Diagnostics`: failures carry stable codes, truthful blame, and compact
  repair-first wording.
- `AG-07 MCP/CLI Contract Parity`: both transports expose the same semantics and visible
  text contract.
- `AG-08 Documentation Truthfulness`: stable docs, tool docs, tests, and examples agree
  on the implemented runtime.

## Criteria Families

| Family | Core readiness question | Minimum evidence expectation |
| --- | --- | --- |
| `GRAM` | Is the authored surface exact, structurally closed, and no-write on malformed input? | `E1`, `E2`, `E5`; plus `E3` for truncation-specific failures |
| `SEL` | Do source and target selections use double-lock validation and contiguous denotation? | `E1`, `E2`, `E3`, `E5` |
| `SAME` | Do same-file programs use one original snapshot, reject overlap, and translate legal moves correctly? | `E1`, `E2`, `E3`, `E5` |
| `BYTE` | Are source bytes, separators, mixed newlines, and empty lines preserved faithfully, while EOF-final-line transfer edge cases normalize target-side line boundaries instead of concatenating lines? | `E1` where semantic wording matters; `E2` and `E5` with raw-byte assertions plus boundary-normalization coverage |
| `ATOM` | Are commit units grouped, planned, rolled back, and summarized correctly? | `E1`, `E2`, `E3`, `E4`, `E5` depending on visible outcome surface |
| `DIAG` | Are diagnostic families, blame hierarchy, partial-success accounting, and repair-first inline behavior stable? | `E1`, `E3`, `E4`, `E5` |
| `PAR` | Do MCP and CLI expose the same success/failure, path anchoring, and host-audit boundary? | `E1` for anchoring rules, `E4`, `E5` |
| `DOC` | Are stable docs, tool docs, and examples synchronized before the tool is called complete? | `E1`, `E5` |

## Minimum Gate Obligations

- Grammar gate: exact envelope/header spellings, structurally valid action shapes,
  explicit side-specific omission tokens, and strict rejection of horizontal truncation.
- Selection gate: double-lock validation, contiguous interval denotation, monotonic line
  numbering, range-based anchored targets, and narrow target-existence rules.
- Same-file gate: one original snapshot, no intermediate-state reinterpretation, overlap
  rejection, correct non-overlapping move translation, and deterministic same-file copy.
- Byte gate: raw-byte equality for ordinary transferred intervals, mixed-newline stability, no trimming or reconstruction of empty/whitespace lines, and targeted result-side line-boundary normalization when an EOF-final-line source selection would otherwise concatenate target text.
- Atomicity gate: alias-aware connected-unit grouping, full planning before writes,
  rollback under write failure, correct full-success / partial-success / failure status,
  and family-compatible `A/M/D` summaries.
- Diagnostics gate: stable minimum code family, truthful blame hierarchy, compact
  `error[CODE]:` headlines, evidence-based optional target anchors, partial-success
  repair accounting, and no audit-shaped tool-managed sidecars.
- Parity gate: success/failure text parity across MCP and CLI, identical workspace/path
  anchoring semantics, transport-consistent host-audit boundary, and family-consistent
  warning behavior if warnings exist.
- Documentation gate: stable spec and tool docs updated before merge, explicit
  source-of-truth precedence after implementation exists, synchronized doc updates for
  behavior changes, and examples that map to tested behavior.

## Minimum Negative-Test Inventory

- malformed envelope or header spellings
- invalid action shapes and malformed selection delimiters
- duplicate or descending numbering and omission misuse
- horizontal truncation markers in selections
- separate source-mismatch and target-mismatch failures
- missing target cases for anchored actions
- same-file intermediate-state dependence and overlap failures
- same-file move translation edge cases
- write-stage rollback failure during multi-path commit
- alias-path grouping and partial success across disjoint units
- host-audit-boundary checks and MCP/CLI parity failures

## Regression Requirements

- every accepted semantic rule has at least one positive test and one nearest-boundary
  negative test
- every user-visible failure class has presentation assertions
- every transport-visible behavior has parity coverage
- the full action basis is reviewable as explicit inventory rather than scattered guesswork
- transfer-fidelity coverage uses raw-byte assertions for ordinary paths in addition to dedicated EOF-final-line boundary-normalization checks
- partial-success coverage asserts both rendered output and resulting filesystem state
- example snippets used in docs remain traceable to tests

## Definition Of Done

`apply_splice` is ready for the target gate only when all top-level goals are satisfied,
every criteria family has explicit pass evidence, the negative suite and permanent
regression obligations pass in CI, stable docs and tool docs are current, representative
MCP/CLI parity cases are green, and review can explain any remaining v1 exclusions as
intentional boundaries rather than accidental gaps.
