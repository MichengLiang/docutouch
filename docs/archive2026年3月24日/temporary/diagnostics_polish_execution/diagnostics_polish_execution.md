# Diagnostics Polish Execution Material

## Status

- Promoted from draft form on 2026-03-22 inside the diagnostics polish execution workspace.
- This document operationalizes `docs/diagnostics_polish_spec.md`.
- This is execution material for review and supervision, not a date-based schedule.
- Superseded in part on 2026-03-23 by the accepted direction that keeps audit/report artifacts rejected but allows failed patch source persistence for model repair.

## 1. Purpose

The purpose of this document is to turn the diagnostics polish specification
into a reviewable execution surface.

It exists to answer:

1. what each execution stage is trying to prove
2. what must be true before a stage counts as complete
3. what evidence the main agent should require before accepting that stage
4. what false progress signals or regression signals should block sign-off

## 2. Explicit Exclusions

This document intentionally excludes:

- calendar dates
- person-by-person staffing narratives
- loose prioritization prose
- “quick wins” framing
- speculative implementation details not grounded in the promoted spec

If a reader wants a roadmap, this is the wrong document.
If a reader wants to know whether a stage is genuinely complete, this is the
right document.

## 3. Governing Inputs

This execution document is downstream of:

- `docs/diagnostics_polish_spec.md`
- `docs/temporary/diagnostics_dx_repair_program.md`

Where this document conflicts with the promoted spec, the promoted spec wins.

## 4. Execution Model

The diagnostics polish effort is complete only if all four execution layers stay
aligned:

1. design contract
2. runtime behavior
3. automated protection
4. documentation / operator trust

Any stage that improves one layer while degrading another should be treated as
incomplete.

## 5. Stage Map

The current execution model is decomposed into six stages:

1. contract lock
2. runtime shaping
3. contract hardening in tests
4. CLI black-box confirmation
5. documentation trust cleanup
6. closure and residual-risk capture

These are not calendar phases.
They are proof phases.

## 5.1 Stage State Legend

Use the following stage states when the main agent or delegated workers update
review status around this document:

- `not_started`
- `in_progress`
- `ready_for_review`
- `accepted`
- `blocked`
- `superseded`

These state labels exist to support review discipline, not project-management
ceremony.

## 5.2 Current Readiness Snapshot

Current checkpoint:

- Stage 1 `contract lock`: `accepted`
- Stage 2 `runtime shaping`: `accepted` for the compact-failure, dedup, empty-patch-unification, and first high-value source-span sharpening wave
- Stage 3 `contract hardening in tests`: `accepted` for the currently touched failure families, including non-context compact failures, empty patch, the first sharper source-span mismatch slice, the latest CLI contract cases, and the narrow help-ownership hardening slice; still open for broader expansion
- Stage 4 `CLI black-box confirmation`: `accepted` for the recent manual matrix and further converted into durable automated protection for high-value patch CLI forms such as no-op positional patch-file, `--patch-file` with spaced targets, and move/write failure rollback behavior
- Stage 5 `documentation trust cleanup`: `accepted` for the current audit-language sweep, live/temporary precedence cleanup, and empty-patch public-doc sync; still open for future historical cleanup if new drift appears
- Stage 6 `closure and residual-risk capture`: `accepted`

Stage 6 acceptance evidence:

- `docs/temporary/diagnostics_polish_execution/diagnostics_polish_closeout.md`

This snapshot is not a roadmap.
It is a checkpoint on what kinds of proof already exist and what remains open by
design.

## 6. Stage Details

### Stage 1: Contract Lock

#### Purpose

Freeze the design answers that should no longer be reopened casually.

This stage exists to prevent later implementation or review work from quietly
backsliding into:

- audit-shaped verbosity
- symmetry-driven output inflation
- fake precision
- prose cleanup that weakens evidence

#### Must be true before complete

- the promoted diagnostics polish spec exists and is readable as a live
  authority
- the promoted spec clearly distinguishes:
  - worth closing
  - worth closing narrowly/selectively
  - do not close
- the spec explicitly names the local maximum for this subsystem
- the spec explicitly excludes prioritization and scheduling concerns

#### Required review evidence

- a promoted live spec in `docs/`
- a reusable judgment rubric
- a compact candidate register or equivalent decision surface
- explicit statements of what should not be reopened

#### False progress signals

- a long “future directions” list with no judgments
- a document that sounds thoughtful but never says “do not close”
- a spec that reintroduces roadmap language
- a spec that lists ideas without defining the local maximum

#### Regression signals

- revived arguments for tool-layer audit persistence
- renewed pressure to flatten partial success for the sake of tidiness
- renewed tolerance for repeated `help:` guidance

### Stage 2: Runtime Shaping

#### Purpose

Make the runtime-visible diagnostics behavior match the locked contract.

This stage covers changes whose main value is user-visible DX:

- compact single full failure
- distinct partial-success accounting
- specific `caused by:` output
- deduplicated strategy guidance
- elimination of convenience-shaped exceptions where the contract should be
  unified

#### Must be true before complete

- ordinary single full failure remains compact by default
- partial success remains heavier where repair accounting is genuinely required
- `caused by:` adds information beyond the headline in the targeted paths
- equivalent `help:` guidance does not repeat across layers
- any new unification, such as empty-patch treatment, stays truthful and does
  not invent fake spans

#### Required review evidence

- focused code diffs in the relevant renderer/runtime surfaces
- before/after failure examples for the touched failure families
- explicit explanation of why the new shape improves repair-turn economics

#### False progress signals

- changed wording without contract-level behavior change
- “cleaner” output that hides low-level cause detail
- broader container blocks that merely look structured
- refactors that move logic around without changing external DX

#### Regression signals

- `failed file groups:` returning to ordinary single full failure
- top-level summaries paraphrasing the headline again
- new failure classes escaping into ad hoc plain-text branches without review

### Stage 3: Contract Hardening In Tests

#### Purpose

Convert the desired DX contract into stable regression protection.

This stage exists because behavior is not really locked until tests can reject
backsliding in:

- information density
- compact-vs-heavy shape discipline
- cause specificity
- CLI/MCP parity where parity matters

#### Must be true before complete

- high-value failure families have direct assertions for the relevant DX rules
- the tests check more than block presence
- compact single full failure is protected across more than one error code
- partial-success accounting remains directly protected

#### Required review evidence

- updated or added tests with assertions such as:
  - help appears once
  - `failed file groups:` absent for compact full failure
  - specific low-level cause present
  - partial-success hints remain distinct
- passing relevant crate test suites

#### False progress signals

- adding tests that only assert the existence of a block header
- covering only `MATCH_INVALID_CONTEXT` and claiming the family is protected
- parity tests that normalize away the very differences the contract cares about

#### Regression signals

- a new failure family ships with no contract-shape assertions
- tests only check codes and ignore message density
- CLI and MCP start drifting for the compact single-full-failure path

### Stage 4: CLI Black-Box Confirmation

#### Purpose

Verify that the public executable surface behaves as the design contract claims,
not merely as internal tests assume.

This stage treats the CLI as a user-facing contract and checks it with
black-box methods such as:

- equivalence classes
- boundary values
- error guessing
- workflow/state coverage
- small combinational coverage where flag forms or path forms matter

#### Must be true before complete

- the high-value CLI paths have been exercised externally
- compact single full failure is confirmed in executable behavior
- partial success is confirmed to retain heavier repair accounting
- the path forms most likely to drift are checked
- any remaining special cases are explicitly surfaced as design decisions, not
  silent surprises

#### Required review evidence

- a black-box matrix with scenarios covered
- command forms exercised, such as:
  - stdin patch
  - positional patch-file
  - `--patch-file`
- captured behavior summaries for:
  - success
  - no-op
  - compact full failure
  - partial success
  - malformed patch
  - missing target families

#### False progress signals

- saying “CLI parity is tested” while never exercising the actual executable
- testing only happy paths
- relying only on internal function behavior to infer public CLI quality

#### Regression signals

- a contract edge remains plain-text while the rest of the family is structured
- path-with-spaces or alternate flag forms drift from expected behavior
- black-box evidence exists once but is never turned into durable protection

### Stage 5: Documentation Trust Cleanup

#### Purpose

Ensure the docs ecosystem teaches the current contract instead of preserving
obsolete mental models.

This stage is complete only when maintainers can trust the docs hierarchy:

- live contract docs teach the current system
- temporary docs do not masquerade as live authority
- historical context remains available without becoming misleading

#### Must be true before complete

- live docs and public tool docs reflect the current compact/heavy failure split
- host-owned audit boundary is taught consistently
- temporary or historical docs are either cleaned, deprioritized, or clearly
  marked
- the most dangerous stale terminology no longer dominates the docs surface

#### Required review evidence

- explicit doc diffs in live specs and public tool docs
- explicit precedence guidance where historical material remains
- targeted searches showing risky old terminology is removed or isolated

#### False progress signals

- polishing only the live spec while leaving misleading temporary docs untouched
- saying “historical docs are fine” when they still read like active guidance
- keeping ambiguous terminology because “the intent is obvious”

#### Regression signals

- readers can still plausibly mistake a temporary plan for live direction
- `auditability` or similar overloaded terms regain normative force
- public tool docs drift away from runtime behavior

### Stage 6: Closure And Residual-Risk Capture

#### Purpose

Conclude a diagnostics-polish cycle without pretending the subsystem is perfect.

This stage exists so the main agent can say:

- what is now locked
- what remains deliberately open
- which apparent rough edges are acceptable by design
- which future work still has real DX value

#### Must be true before complete

- completed gains are stated in contract language, not just implementation terms
- residual risks are explicit
- remaining valuable work is separated from rejected or closed questions
- the subsystem can be described honestly as high-quality, even if not yet at
  the local maximum

#### Required review evidence

- a short closeout statement tied back to the promoted spec
- explicit “do not reopen” questions
- explicit residual-value areas such as:
  - source-span precision improvements
  - broader durable CLI black-box protection

#### False progress signals

- declaring victory without naming residual risk
- reopening settled debates in the closeout itself
- treating “not perfect yet” as evidence that the prior stages failed

#### Regression signals

- lack of any final statement about what is now stable
- future work lists that ignore the spec’s do-not-close judgments
- ambiguity about whether the subsystem is still in redesign mode

## 7. Cross-Stage Acceptance Matrix

The following matrix summarizes how the main agent should judge completion:

| Stage | Completion marker | Evidence expected | Main rejection trigger |
| --- | --- | --- | --- |
| Contract lock | live spec is decisive and bounded | promoted spec, rubric, candidate register | design doc drifts into roadmap or wishlist |
| Runtime shaping | external behavior matches contract | targeted diffs and before/after failure examples | wording churn without DX change |
| Test hardening | contract survives regression pressure | high-value assertions plus green suites | tests still check only block presence |
| CLI black-box confirmation | public executable contract is verified | scenario matrix and observed behavior | internal confidence used as CLI substitute |
| Documentation trust cleanup | docs hierarchy becomes trustworthy | live-doc sync plus precedence guidance | temporary docs still masquerade as authority |
| Closure and residual-risk capture | finishable truth is recorded | closeout statement and explicit residuals | fake completeness or reopened settled debates |

## 8. Review Heuristics For The Main Agent

When supervising sub-agents, the main agent should repeatedly ask:

1. does this stage produce reviewable proof, or just motion?
2. if this text/code/test disappeared, what contract guarantee would vanish?
3. is the result better for the model, or just nicer for the maintainer to read?
4. is the output becoming more unified, or merely more uniform?
5. is this stage closing a real DX gap, or only consuming attention?

## 9. Common False-Completion Patterns

The following patterns should not count as completion:

- “the message looks cleaner now” without evidence of repair-turn gain
- “tests pass” when the tests never asserted the claimed contract
- “docs were updated” when historical docs still mislead readers
- “CLI probably matches” without black-box evidence
- “the system is consistent” when special cases are simply being ignored

## 10. Minimal Review Packet For Any New Polish Cycle

Before the main agent signs off on any future polishing wave, the review packet
should contain:

1. the specific contract claim being strengthened
2. the targeted failure families or docs surface
3. the evidence that the change improves LLM-facing DX
4. the tests or black-box checks that now protect it
5. the false-progress or regression risks that were checked

## 11. Final Execution Posture

This execution material assumes the subsystem is already in a high-quality band.

The goal is therefore not broad reinvention.
The goal is disciplined finishing work with tight review discipline.

That means:

- no reopening closed questions without new evidence
- no turning execution material back into speculative design
- no confusing activity with proof

The right outcome of this workspace is a review process that can tell the
difference between:

- real diagnostics polish
- cosmetic churn
- and accidental regression hidden inside “cleanup”
