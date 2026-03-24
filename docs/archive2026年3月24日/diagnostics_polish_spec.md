# Diagnostics Polish Spec

## Status

- Promoted from the temporary diagnostics polish workspace on 2026-03-22.
- This is the active design specification and judgment framework for diagnostics polishing in the Rust DocuTouch project.
- It assumes the recent compact-single-full-failure, help-dedup, docs-sync, and black-box work has already landed.

## Explicit Exclusion

This workspace intentionally excludes:

- prioritization
- sequencing
- scheduling
- milestone planning
- task breakdown

If a reader wants to know what should happen first, that belongs somewhere else.
This document is only for deciding what the diagnostics system should ideally be,
what remains worth polishing, and what should be left alone.

## 1. Problem Framing

The question is no longer “is the diagnostics system broken?”

That question has already been answered.
The main redundancy regression was real, and the current implementation is now materially better.

The right design question is:

> What remaining polishing is still worth doing if the goal is the best LLM-facing diagnostics contract this scope can realistically support?

That question has two parts:

1. Which gaps are still real DX losses?
2. Which apparent imperfections are not worth touching because “more polishing” would actually create churn, verbosity, or architectural confusion?

This document answers both.

## 2. North Star

The diagnostics system should help ChatGPT/Codex-class models recover from failure with:

- the fewest wasted tokens
- the fewest wasted repair turns
- the strongest truthful blame signal the runtime can actually justify

The local target is therefore:

- compact default failure output
- strong information density
- explicit repair accounting when partial success exists
- full committed / failed path visibility when partial success exists
- one obvious next move
- no audit-shaped repetition inside the tool layer
- no fake precision

## 3. Judgment Rubric

No proposed diagnostics polish should be accepted merely because it sounds
cleaner, more uniform, or more sophisticated.

Any proposed change should pass the following reusable gate set.

### Gate 1: Repair-turn gain

Question:

- does this change reduce the number of wasted repair turns for the model?

If not, it starts with a presumption against change.

### Gate 2: Information-density gain

Question:

- does this change increase useful information per token, or at least preserve
  it while simplifying the contract?

If the change mostly adds prose, repetition, or decorative structure, reject it.

### Gate 3: Truthfulness

Question:

- is the runtime genuinely entitled to say this?

If a change depends on fake precision, speculative blame, or misleading
summary language, reject it.

### Gate 4: Contract unification

Question:

- does this change reduce unnecessary special cases in the public diagnostics
  contract?

This gate favors unification when unification improves model understanding.
It does not justify flattening meaningful distinctions such as the heavier
partial-success accounting shape.

### Gate 5: Maintenance sanity

Question:

- does this change make future drift less likely without forcing churn for its
  own sake?

If a change creates naming churn, taxonomy churn, or architectural complexity
without durable contract benefit, reject it.

### Gate 6: Host-boundary discipline

Question:

- does this change respect the host-owned audit boundary?

Any change that reintroduces tool-layer audit theater, or justifies extra text
for imagined audit consumers, should be rejected.

This gate does not reject failed patch source persistence when that persistence
creates a stable repair object for the model and does not turn into a second
reporting subsystem.

### Gate 7: Common-path discipline

Question:

- does this change keep the ordinary path boring?

Single full failure should remain compact by default.
Any expansion of the default path must prove that it changes the next repair
move, not just the look of the output.

### Decision rule

In practice:

- a change is **worth closing** only if it clearly passes Gates 1, 2, and 3,
  and does not fail Gates 4 through 7
- a change is **selectively worth closing** if it helps only a narrow class of
  failures or documents, and should therefore be applied with explicit scope
  limits
- a change is **do not close** if it mainly improves aesthetics, symmetry, or
  theoretical neatness without strong repair-loop benefit

## 4. Ideal Contract

The ideal contract for this subsystem is:

### 4.1 Headline

Every failure begins with a stable code-bearing headline that tells the model what class of failure occurred.

### 4.2 Blame location

Every failure points to the smallest truthful patch-side location the runtime can robustly support.

When a target-side anchor materially improves repairability and the runtime has strong corroborating evidence, include one compact target anchor.

### 4.3 Cause

`caused by:` should add real information.

It should prefer:

- the most specific low-level cause that is still readable enough
- not a generic paraphrase of the headline

### 4.4 Guidance

`help:` should express the minimum next repair move.

It should appear:

- once per distinct strategy
- never as layer-shaped repetition

### 4.5 Shape discipline

Single full failure should stay compact.

Partial success should stay heavier, because it must preserve repair accounting for:

- what committed
- what failed
- what was attempted

That heavier shape should enumerate the relevant committed and failed paths in
full. Compactness should come from less prose, not from hiding repair-critical
path accounting.

### 4.6 Product boundary

Durable audit belongs to the host, not the tool.

The tool should carry repair-relevant accounting and may persist the failed patch
source itself when the original patch was not already file-backed. It should not
grow second-layer audit theater, replay caches, or secondary JSON reports.

## 5. Current Contract

After the recent work, the current system is substantially improved:

- duplicate `help:` output in the primary failure path is suppressed
- single full failure no longer automatically borrows partial-failure bulk
- `caused by:` now keeps more specific low-level messages in the main mismatch paths
- empty patch now joins the same structured diagnostics family without fake spans
- high-value update-chunk mismatch failures can now blame the first concrete expected old line instead of only the `@@` marker when that is the truthful locus
- public docs, internal specs, and tests are more aligned around repair accounting
- CLI and MCP parity now explicitly covers compact single full failure
- non-context full-failure codes now have better regression protection
- the most failure-prone patch CLI forms now have broader durable coverage, including no-op positional patch-file and `--patch-file` with spaced targets
- the mirrored-help ownership risk is reduced because top-level `FailureDetails` no longer reuses per-unit `help` as if it were top-level guidance
- historical temporary docs are less likely to teach the old audit-heavy mental model

The current system is therefore no longer in “obviously regressed” territory.

It is in “strong but not saturated” territory.

## 6. Remaining Gaps

The remaining gaps are real, but they are no longer all the same kind of problem.

They split into four categories:

### 6.1 Contract gaps

- some documentation still frames limitations more clearly than resolutions, which is honest but means the contract is not yet fully crystallized as one family

### 6.2 Precision gaps

- some execution failures still stop at coarse action/hunk anchors rather than richer source-span-grade evidence
- target-side evidence remains thin in some mismatch scenarios
- only the primary failed unit consistently surfaces a patch-side location; later failed groups do not yet render their own patch pointers or anchors

### 6.3 Coverage gaps

- current automated coverage is strong for high-value cases, but still curated rather than systematic across the full failure taxonomy
- CLI black-box evidence exists, but not all of it is yet durable automated protection

### 6.4 Structural gaps

- some shaping is still achieved in presentation rather than by a fully clarified cross-layer ownership model
- partial-failure presentation still compresses some committed lists and still leaves multiline failed-group causes under-indented or structurally uneven

This last point is not a current user-facing bug.
It is a future-maintenance risk.

## 7. Candidate Register

This register is the compact decision surface for the current polish space.
It exists so future readers do not have to infer status from prose.

| Candidate | Classification | Why |
| --- | --- | --- |
| Empty patch unification | Closed in the current contract | The public contract no longer has this convenience-shaped branch |
| Push more execution failures toward source-span-grade diagnostics | Worth closing | Directly reduces wasted repair turns |
| Add richer multi-span rendering broadly | Not broadly worth closing right now | High risk of rustc-cosplay churn without proportional DX gain |
| Compress partial success further | Do not close | Would sacrifice necessary repair accounting |
| Replace raw low-level causes with cleaner prose | Do not close | Risks lower information density |
| Turn CLI black-box matrix into stable automated coverage | In progress with strong recent gains | The highest-value patch CLI forms are now better protected, but not the whole public matrix |
| Complete taxonomy unification | Worth closing, but narrowly | Valuable for teachability, risky if it turns into naming churn |
| Mark historical docs more aggressively | Worth closing selectively | Reduces documentation trust hazards |
| Preserve tool-layer audit concepts just in case | Do not close | Reopens a rejected product direction |

## 8. Candidate Improvements and Judgment

This section is the core of the spec.
Each candidate is judged explicitly.

### Candidate A: Unify empty patch into the structured diagnostics family

Current state:

- empty patch now renders as a small structured outer-format failure
- CLI and MCP both flow through the shared patch renderer

Judgment:

- **Closed in the current contract**

Why:

- this is the clearest remaining “special case by convenience”
- it creates unnecessary mental branching in the public contract
- it conflicts with the emerging “one best failure style” philosophy

Constraint:

- do not invent fake spans if none exist
- if it joins the family, it should do so honestly, likely as a small structured parse/preflight failure rather than a theatrical compiler block

### Candidate B: Push more execution failures toward source-span-grade diagnostics

Current state:

- some failures still land on coarse action/hunk metadata

Judgment:

- **Worth closing**

Why:

- this is the next highest-value DX improvement after redundancy removal
- it reduces wasted repair turns more than additional wording polish would

Constraint:

- truth over prettiness
- no fake spans
- prefer one solid primary span over speculative multi-span noise

### Candidate C: Add richer multi-span rendering broadly

Current state:

- the system currently stays conservative and mostly single-anchor

Judgment:

- **Not broadly worth closing right now**

Why:

- it would be easy to slide from repair signal into rustc cosplay
- the subsystem’s biggest remaining weakness is not “too few spans everywhere”; it is “too few truthful spans in specific high-value cases”

Refined position:

- selective, evidence-backed additional anchors can be worth it
- broad multi-span expansion as a general aesthetic upgrade is not worth it

### Candidate D: Fully enumerate partial success accounting

Current state:

- partial success intentionally preserves more accounting than single full failure
- the current renderer still compresses some committed lists and still mixes transaction summary with first-failure-local presentation

Judgment:

- **Worth closing**

Why:

- the heavier shape is not ornamental
- safe recovery depends on seeing every committed and failed path, not an omission sentence
- the next repair turn is materially better when the model can reread the exact patch object and all committed / failed paths

Constraint:

- compress prose, not repair-critical path accounting
- keep the common single-full-failure path compact

### Candidate E: Remove raw low-level causes and replace them with cleaner prose

Current state:

- the system now prefers more specific raw low-level causes in key paths

Judgment:

- **Do not close**

Why:

- this would regress information density
- the recent fixes moved in the correct direction already

Refined position:

- improve readability only when it does not destroy specificity
- never replace useful evidence with abstract prose for style reasons

### Candidate F: Turn CLI black-box matrix into stable automated coverage

Current state:

- recent black-box runs produced strong confidence signals
- not all high-value black-box cases are yet encoded in durable automated checks

Judgment:

- **In progress with strong recent gains**

Why:

- this is one of the cheapest ways to protect the public contract
- it keeps CLI-visible drift from sneaking past code-centric tests

### Candidate G: Complete taxonomy unification

Current state:

- the system is much more coherent than before, but not yet fully systematized as one diagnostics family

Judgment:

- **Worth closing, but narrowly**

Why:

- the right target is clearer ownership and more predictable grammar
- the wrong target is endless renaming or category churn

Constraint:

- unify for teachability and maintenance
- not for diagram beauty

### Candidate G1: Narrow help-ownership hardening

Current state:

- top-level `FailureDetails` no longer mirrors the first failed unit's `help`
- compact single full failure now lifts failed-unit help only at render time when no genuine top-level guidance exists

Judgment:

- **Closed in the current contract as a narrow hardening step**

Why:

- it reduces a real future-regression risk without destabilizing current visible DX
- it keeps per-unit guidance owned by `failed_units`
- it avoids a broad renderer or taxonomy rewrite

### Candidate H: Keep historical docs but mark them more aggressively

Current state:

- historical docs are safer than before, but some still require readers to understand precedence

Judgment:

- **Worth closing selectively**

Why:

- historical docs can still mislead future implementation work
- this is a documentation trust issue, not merely a wording issue

Constraint:

- keep useful history
- make live-contract precedence unmistakable

### Candidate I: Preserve tool-layer audit concepts “just in case”

Current state:

- the host-audit boundary is now much clearer

Judgment:

- **Do not close**

Why:

- this is not a missing feature
- it is a direction the product explicitly rejected

Any move back toward tool-layer audit persistence would be negative polishing.

Failed patch source persistence for model repair is not the same thing as tool-
layer audit persistence and should not be rejected under this candidate.

## 9. Apparent Imperfections That Should Be Left Alone

These points matter because the team is now at a stage where over-polishing is a real risk.

### 9.1 Partial success is allowed to be heavier

That is not inconsistency.
That is correct modeling of repair risk.

The remaining polish work is to make that heavier path fully enumerated and more
structurally regular, not to flatten it back down.

### 9.2 Raw low-level causes are sometimes ugly

Ugly but truthful is often better than polished but weaker.

### 9.3 Not every failure needs multi-span treatment

The goal is repairability, not compiler aesthetics.

### 9.4 A little asymmetry between failure classes is acceptable

Uniformity is good only when it improves model understanding.
Forced symmetry that hides causal differences is bad DX.

## 10. Local Maximum Definition

The local maximum for this subsystem is **not** “perfect diagnostics in the abstract.”

The local maximum is the best diagnostics contract this Rust DocuTouch scope can support without becoming heavier, faker, or more expensive to maintain than its value justifies.

That local maximum would look like this:

### 10.1 Contract

- all common patch failures belong to one coherent diagnostics family
- empty patch no longer feels like an arbitrary outsider
- durable audit remains fully host-owned
- failed inline patches can materialize as stable patch-source files for the next repair step without growing a second audit subsystem

### 10.2 Precision

- most high-value execution failures reach the best truthful source-span currently available
- selected mismatch families can use one compact corroborating target anchor where it genuinely helps

### 10.3 Density

- single full failure stays compact
- partial success stays fully repair-accounted
- partial success does not compress committed / failed path accounting behind omission prose
- no duplicated strategy guidance
- no final summary that merely rewords the headline

### 10.4 Verification

- automated tests protect the contract across the most important failure families
- CLI black-box behavior is protected for the public-facing paths most likely to drift

### 10.5 Docs

- live docs teach the same contract the runtime and tests enforce
- historical docs remain readable but cannot plausibly be mistaken for live direction

## 11. What Still Needs Real Design Work

Based on the judgment calls above, the remaining legitimate design work is:

1. sharpen source-span-grade execution diagnostics in the highest-value failure families
2. fully align partial-failure presentation around full enumeration, stable patch pointers, and multiline cause indentation
3. convert the black-box confidence matrix into stable contract protection
4. tighten taxonomy and ownership boundaries where future drift is still plausible

## 12. What Does Not Need More Design Work

The following should not be reopened unless new evidence appears:

1. whether single full failure should be compact by default
2. whether repeated `help:` lines are acceptable
3. whether the tool should keep its own durable audit layer
4. whether partial success should be flattened into a lighter-but-riskier summary
5. whether generic prose is preferable to specific low-level causes

These questions are no longer open design questions.
They already have correct answers for this subsystem.

## 13. Final Assessment

The subsystem is already in the high-quality band.

It is not yet at the local maximum.
But the remaining headroom is no longer broad conceptual confusion.
It is concentrated, finishable, and mostly about:

- unification
- precision
- contract hardening

That is the correct state for a system that has moved past obvious DX failure and into genuine polishing territory.
