# `apply_splice` Review Log

## Round 0

- Created closure workspace.
- Independent design-pass agent launched without inherited context.
- Awaiting first proposal for review and QA.

## Round 1

- Received first independent proposal.
- Main strengths identified:
  - correctly treats `apply_splice` as a structural-operation primitive over existing spans
  - keeps the public product boundary separate from `apply_patch`
  - locks the shared-substrate direction instead of reopening vendored-merge debates
  - pushes canonical grammar, selection relation, partial-success semantics, and
    diagnostics obligations in the right direction
- Main QA gaps identified for the next pass:
  - target-range operational semantics need a more exact mapping to insertion and
    replacement points
  - same-file multi-action semantics need a formal rejection of
    intermediate-state-dependent interpretation
  - byte-span and newline preservation need a tighter definition
  - grammar edge cases need explicit invalid-shape rules
  - runtime policy details such as parent-directory creation and `A/M/D`
    computation need a crisper contract
  - a recursive test model is still missing

## Round 2

- Sent a targeted refinement request asking for:
  - deeper target-range semantics
  - stronger same-file snapshot rules
  - explicit byte-span/newline semantics
  - grammar edge-case closure
  - runtime policy details
  - a recursive QA/test decomposition

## Round 3

- During QA, identified a baseline drift item:
  - current `codex-apply-patch` implementation rewrites move outcomes to
    destination-side `modified` entries
  - `docs/apply_patch_semantics_plan.md` still shows an example with `M from.txt`
- Sent a reconciliation request asking the independent agent to:
  - resolve the `Move` summary baseline for `apply_splice`
  - propose a source-of-truth precedence rule for closure review
  - produce a compact decision register separating closed decisions,
    representation gaps, and observed drift items

## Follow-up

- Reconciled the move-summary baseline against:
  - `docutouch-server/tool_docs/apply_patch.md`
  - `codex-apply-patch/src/lib.rs`
  - `codex-apply-patch/tests/suite/tool.rs`
- Confirmed the runtime baseline is destination-side `M` for move-shaped success
  summaries.
- Fixed the stale `M from.txt` example in
  `docs/apply_patch_semantics_plan.md`.
- Launched two parallel follow-up workers:
  - one for formal grammar and execution semantics draft
  - one for architecture / diagnostics / source-of-truth / test-model draft
- Later governance pass:
  - one worker produced detailed schedule / phase-gate draft
  - one worker produced detailed acceptance / QA criteria draft
  - main-agent completed cross-review and added governance consistency notes
