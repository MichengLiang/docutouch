# `apply_patch` Contract Repair Execution Plan

## Status

- Recorded on 2026-03-23.
- Executed on 2026-03-23 for the current local interface contract.
- Scope: execution plan and closeout record for already-confirmed `apply_patch`
  contract repairs.
- Authority level: temporary implementation-and-closeout artifact.

## Purpose

The purpose of this document is narrow and explicit:

- convert already-confirmed `apply_patch` contract defects into a repair program
- preserve separation between current-interface repairs and future-interface
  design work

This document exists because the current discussion has already produced owner-
confirmed repair directions. Those directions should not remain trapped in chat
history.

## Inputs

This plan is downstream of:

- `docs/temporary/apply_patch_confirmed_gaps_20260323.md`
- `docs/temporary/apply_patch_anchor_semantics_investigation.md`

## Confirmed Repair Set

The current repair set contains six items.

1. allow empty `Add File`
2. preserve existing-file newline style during update
3. preserve existing-file EOF newline state during update
4. remove active prompt/runtime drift around stacked multiple `@@`
5. repair standalone transport parity debt
6. add regression protection for the repaired contract

## Workstream R1: Empty `Add File`

### Objective

Change local behavior so empty `Add File` is no longer rejected as malformed.

### Required changes

- parser behavior
- success-path or warning-path behavior
- active tool docs
- tests that currently assert failure

### Acceptance condition

- empty file creation succeeds under the chosen local contract
- user-visible messaging reflects the intended local guidance

## Workstream R2: Newline Preservation for Existing Files

### Objective

Stop update application from rewriting existing-file newline style.

### Required changes

- update-content transformation path in `codex-apply-patch`
- regression tests for CRLF-preserving updates
- docs reflecting the contract

### Acceptance condition

- updating an existing CRLF file no longer produces mixed newline output as a
  side effect of the tool

## Workstream R3: EOF Newline Preservation for Existing Files

### Objective

Stop update application from implicitly appending a trailing newline to existing
files whose pre-mutation state lacked one.

### Required changes

- final output reconstruction logic in the existing update pipeline
- regression tests for EOF-without-newline preservation
- docs reflecting the contract

### Acceptance condition

- the tool no longer changes EOF newline state of existing files unless the patch
  itself semantically requires that change

## Workstream R4: Active `@@` Prompt/Runtime Drift Repair

### Objective

Remove or downgrade active prompt teaching that exceeds the current parser-
backed contract.

### Required changes

- active injected tool docs
- wording around multiple `@@`
- examples that currently imply unsupported forms

### Acceptance condition

- the active prompt-facing doc no longer teaches stacked multiple `@@` as a
  normal supported path under the current parser

## Workstream R5: Standalone Transport Parity Repair

### Objective

Repair the current drift between the healthier DocuTouch transport contract and
the vendored standalone `apply_patch` CLI surface.

### Required changes

- standalone tests that still encode outdated outward behavior
- user-visible success and failure messaging where applicable

### Acceptance condition

- standalone behavior is either aligned with the chosen local contract or
  explicitly documented as intentionally different

Current owner direction favors repair rather than indefinite tolerance.

## Workstream R6: Regression Hardening

### Objective

Ensure the repaired contract is protected against silent regression.

### Required test coverage

1. empty `Add File`
2. CRLF-preserving update behavior
3. mixed-newline non-corruption under the chosen contract
4. EOF-without-newline preservation
5. active `@@` contract expectations
6. repaired transport parity cases

### Acceptance condition

- the repaired behavior is encoded in durable tests rather than only in docs or
  ad hoc manual memory

## Execution Order

The execution order is fixed as follows.

1. update active tool docs for current `@@` drift so prompt harm stops first
2. repair empty `Add File`
3. repair newline style preservation
4. repair EOF newline preservation
5. repair standalone parity debt
6. finalize regression hardening and closeout docs sync

## File Impact Surface

The following files are likely to be touched during execution.

- `codex-apply-patch/src/parser.rs`
- `codex-apply-patch/src/lib.rs`
- `docutouch-core/src/patch_runtime.rs`
- `docutouch-core/src/patch_presentation.rs`
- `docutouch-server/tool_docs/apply_patch.md`
- relevant historical tool doc variants where drift would remain dangerous
- `codex-apply-patch/tests/suite/tool.rs`
- `docutouch-server/tests/stdio_smoke.rs`
- any additional targeted tests in `docutouch-core` or `codex-apply-patch`

## Closeout Condition

This repair plan is complete only when all of the following are true.

1. the active prompt-facing contract no longer teaches unsupported stacked `@@`
2. empty `Add File` follows the new local contract
3. existing-file newline and EOF state are no longer silently reformatted
4. standalone transport debt is no longer left as silent drift
5. the repaired contract is protected by durable regression coverage

## Execution Closeout

The execution described above has now been carried through the current local
surface.

Closed outcomes:

1. empty `Add File` is accepted under the local contract
2. existing-file update no longer rewrites newline style as a side effect
3. existing-file update no longer forces an EOF newline when the patch does not
   semantically require one
4. active prompt-facing `@@` guidance has been narrowed to the parser-backed
   contract
5. server-facing regression coverage now asserts the repaired contract instead of
   preserving the earlier defects

Residual note:

- The vendored raw standalone `apply_patch` binary remains a separate transport
  surface. Its outward UX should be kept aligned, but that concern is distinct
  from whether the core repair set above has landed in the shared local
  contract.

## Relationship to Future Interface Work

This repair plan intentionally does not absorb the future line-locked
`apply_patch` extension design effort.

That future work should proceed in parallel as a distinct design and evaluation
track inside the same tool family. The current plan exists to make the current
interface contract healthy enough that the eventual comparison of locking
strategies is fair.
