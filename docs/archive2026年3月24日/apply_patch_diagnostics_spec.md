# `apply_patch` Diagnostics Detailed Design

## Status

- Detailed design specification
- Intended to guide the next diagnostics-focused implementation wave
- Complements `docs/apply_patch_semantics_plan.md`

## 1. Why this document exists

`docs/apply_patch_semantics_plan.md` already captures the compatibility posture,
warning philosophy, and current product decisions around `apply_patch`.

That document is still the right place for:

- semantics and compatibility rationale
- warning philosophy
- correctness hardening priorities
- historical decision tracking

This document has a different purpose:

- flatten the diagnostics problem into a concrete specification
- describe what the next diagnostics wave should actually build
- separate detailed design from implementation scheduling

In short:

- `apply_patch_semantics_plan.md` explains the product position
- this document explains the target diagnostics system

## 2. Current baseline

The current implementation already provides materially better diagnostics than the
original failure surface.

Current visible strengths:

- stable error codes on many failure classes
- compact rustc-inspired failure rendering
- source excerpts for outer-format failures when source spans are known
- committed `A/M/D` preservation in partial-failure output
- failed file-group enumeration with attempted changes
- targeted `help:` messages for common repair actions
- warning blocks appended to success without replacing the primary success shape
- inline failure messages as the primary repair surface, with optional failed-patch-source persistence when the original patch was not already file-backed

Current limitations:

- execution failures are better, but not yet reliably source-span grade
- many execution-stage failures still anchor at target path or coarse action/hunk
  metadata instead of exact patch-source spans
- target-side evidence is still thin in several high-value mismatch scenarios
- the current docs still contain stale references to an inline-only repair policy that is now too absolute
- partial-failure presentation still compresses some committed-path accounting and still mixes transaction-level and first-failure-local hierarchy
- failed-group multiline cause formatting and per-group patch pointers are not yet fully systematized
- warning and error taxonomy are stronger than before, but not yet fully
  systematized as one coherent diagnostics family

## 3. Core design principle

The target standard is:

> rustc-like honesty, not rustc cosplay.

This means:

- diagnostics should be precise where the runtime truly knows the blame location
- diagnostics should remain compact and strategy-shaping
- diagnostics should not imitate compiler aesthetics at the expense of truth
- the system must never invent a fake span just to look sophisticated

The best diagnostic is not the prettiest one.
The best diagnostic is the one that lets the next repair step succeed with the
fewest wasted turns.

## 4. Product goals

The diagnostics system should optimize for the actual consumer: an LLM trying to
repair or continue work after a failed patch application.

Primary goals:

- make the failing cause visible
- localize the failure to the smallest truthful patch-source span possible
- preserve partial-success repair accounting
- preserve a stable patch object for follow-up repair when the failing patch did not already come from a persistent file
- teach the next repair strategy without turning output into a tutorial
- keep the inline failure message as the primary repair surface
- reject tool-layer audit theater while still allowing failed patch source persistence for the immediate repair loop

## 5. Non-goals

The diagnostics system should not:

- become a decorative terminal art project
- dump large target-side file bodies into the primary message
- show many spans by default just because the implementation can
- replace the Codex success-path summary shape with a custom DocuTouch success
  dialect
- reintroduce tool-managed audit sidecars, replay caches, or secondary JSON reports as a second audit layer

## 6. Diagnostics model

The target diagnostics model should be understood as a layered contract.

### Layer 1. Headline

Every failure should begin with a stable headline:

```text
error[CODE]: human-readable summary
```

The headline should answer:

- what class of failure occurred
- whether the patch failed fully or partially

### Layer 2. Primary blame location

The next line should point to the best truthful primary location.

Preferred order:

1. exact patch-source span
2. target path only when no truthful patch-source span exists
3. patch path only when neither of the above is known

### Layer 3. Minimal evidence block

The message may include:

- one source excerpt around the patch-source span
- one optional target-side anchor when the causal chain is robust
- action / hunk indices when they materially improve repairability

This block exists to prove the diagnosis, not to narrate the entire failure.

### Layer 4. Repair guidance

The message should end with:

- committed changes, when any were applied
- failed file groups, when relevant
- attempted changes, when relevant
- full committed / failed path enumeration whenever that accounting changes the next repair move
- one stable patch-source path when the runtime persisted the failed patch source for follow-up repair
- compact `help:` lines
- no secondary audit/report file requirement for ordinary repair

## 7. Source-span grade execution diagnostics

This is the most important next wave.

### 7.1 Problem

Outer-format parse failures already map well into patch source.
Execution-stage failures still do not consistently do so.

That gap costs repair efficiency because the caller often learns:

- the target path
- the failure class
- the failed action/hunk

but not the smallest truthful patch location that caused the runtime failure.

### 7.2 Target behavior

For any execution failure where a patch action, chunk, or resolved hunk can be
causally tied back to source-map information, the failure should carry:

- `source_line`
- `source_column`
- optionally a bounded source range when the evidence is robust enough

The design requirement is not "all failures must have spans".
The design requirement is:

- when the runtime truly knows the span, surface it
- when the runtime does not know the span, do not fabricate one

### 7.3 Priority failure classes

The first wave should focus on classes where better patch-source mapping has the
highest repair value:

- `MATCH_INVALID_CONTEXT`
- `MATCH_INVALID_EOF_CONTEXT`
- commit-stage path failures that can still be traced to a specific action
- move / alias / path-identity failures once those classes become more explicit

## 8. Optional target-side anchoring

The ideal diagnostics system should support one optional secondary anchor when
it materially improves repair quality.

Examples:

- patch-source span points to the failing update chunk
- target-side anchor points to the current file location where the mismatch or
  conflict became evident

This is intentionally narrower than general multi-span rendering.

Recommended rule:

- default to one primary blame span
- add one secondary target anchor only when the causal chain is strong and the
  extra line clearly helps the next repair step

## 9. Warning and error unification

The long-term system should treat warnings and errors as one diagnostics family,
not as unrelated formatting accidents.

Desired shared properties:

- stable diagnostic codes
- consistent tone
- consistent target-path rendering
- consistent `help:` style
- consistent relationship to runtime truth

This does not mean warnings and errors should look identical.
It means the consumer should experience one coherent diagnostics grammar.

## 10. Host-audit boundary

DocuTouch no longer treats tool-managed audit sidecars as part of the repair contract.

Current product direction:

- the tool returns an inline failure message as the primary repair surface
- the tool may persist the failed patch source itself when the original patch was not already file-backed
- the tool still avoids secondary audit reports, replay caches, and extra JSON payloads for ordinary repair loops
- any broader audit trail belongs to the Codex host, which already has the full
  tool-call request / response history

Implications:

- diagnostics design should spend budget on better inline blame evidence and stable patch references, not on a second audit persistence layer
- any metadata needed for immediate repair should appear directly in the message or in the persisted patch source path itself
- host-level logging may still preserve richer traces, but that is outside the
  tool contract

## 11. Render contract examples

### 11.1 Parse / outer-format failure

```text
error[OUTER_INVALID_ADD_LINE]: Add File block is malformed
  --> input.patch:40:1
   |
40 | *** Add File: docs/example.txt
   | ^
   = action: 1
   = help: prefix each Add File content line with '+' before retrying

caused by:
  Add File block requires lines prefixed with '+'
```

### 11.2 Execution failure with primary patch-source span

```text
error[MATCH_INVALID_CONTEXT]: patch context did not match target file
  --> .docutouch/failed-patches/20260323T214501Z-7f3c.patch:28:1
   |
28 | -    missing = 1
   | ^
   |
   = patch: .docutouch/failed-patches/20260323T214501Z-7f3c.patch
   = target: src/handler.rs
   = action: 1
   = hunk: 4
   = help: re-read the target file and regenerate the patch with fresh context

caused by:
  Failed to find expected lines in src/handler.rs
```

### 11.3 Partial failure with preserved repair accounting

```text
error[PARTIAL_UNIT_FAILURE]: patch partially applied
  --> .docutouch/failed-patches/20260323T214501Z-7f3c.patch:75:1
   |
75 | *** Update File: src/b.rs
   | ^
   |
   = patch: .docutouch/failed-patches/20260323T214501Z-7f3c.patch
   = target: src/b.rs
   = action: 2

   = committed changes:
     [01] A src/a.rs

   = failed file groups: 1

   = failed_group[1]:
     error[TARGET_WRITE_ERROR]: patch could not be written to the target path
     --> src/b.rs
     = patch: .docutouch/failed-patches/20260323T214501Z-7f3c.patch:75:1
     = action: 2
     = attempted changes:
       M src/b.rs
     = caused by:
       Failed to create parent directories for src/b.rs
     = help: repair the target path permissions or filesystem state and retry

   = help: re-read committed files and retry only the failing groups
   = help: do not reapply committed groups unchanged

caused by:
  Patch partially applied.
  1 committed file group succeeded.
  1 failed file group requires repair.
```

These examples are not mandates for exact wording in every case.
They describe the intended structure and salience hierarchy.

## 12. Detailed design requirements

### 12.1 Always preserve truth over style

- never fabricate a span
- never imply stronger causal certainty than the runtime has
- prefer a coarser truthful anchor over a sharper fake one

### 12.2 Keep one obvious next move

Each failure class should have at most a small number of strategy-shaped help
messages.

Good help text:

- tells the caller the minimum next repair move
- does not turn into a full tutorial
- does not duplicate what the headline already said
- does not repeat equivalent guidance already carried by a failed-unit block

### 12.3 Keep partial-failure repair accounting first-class

Partial success is one of the fork's defining semantics.
Therefore diagnostics must always keep clear separation between:

- what was committed
- what failed
- what was attempted

This is more valuable than decorative prose.
It is still a repair-oriented accounting requirement, not a second audit layer
inside the tool.

The current product direction also requires:

- full committed / failed path enumeration when partial success occurs
- no omission prose such as “... and N more committed changes” in the public repair contract
- stable patch-source persistence when failure-time repair needs a readable patch object

### 12.4 Keep the common path boring

Warning-free success must remain terse.
The diagnostics system should become more powerful by improving warning and
failure surfaces, not by destabilizing the main success contract.
Ordinary single full failure should also stay compact unless an expanded failed-
group block changes the next repair move.

Compactness should apply to wording and repeated explanation, not to repair-
critical committed / failed path accounting in partial success.

## 13. Recommended next implementation order

### Wave 1. Source-span grade execution diagnostics

Primary goal:

- carry exact patch-source locations deeper into execution-stage failures where
  the causal mapping is robust

Expected payoff:

- highest immediate reduction in repair turns

### Wave 2. One optional target-side anchor

Primary goal:

- add a single corroborating target anchor for the highest-value mismatch cases

Expected payoff:

- better repairability without large output expansion

### Wave 3. Taxonomy hardening

Primary goal:

- make warning and error code families more systematic

Expected payoff:

- more teachable and more stable diagnostics grammar

### Wave 4. Host-audit boundary cleanup

Primary goal:

- remove stale sidecar assumptions from the diagnostics design while keeping the
  tool contract repair-first rather than audit-shaped

Expected payoff:

- less duplicated audit infrastructure and a clearer division between tool
  diagnostics and host observability

## 14. Acceptance criteria for the next wave

The next diagnostics wave should count as successful only if it satisfies all of
the following:

- execution failures become measurably more likely to point at truthful
  patch-source locations
- no new verbosity explosion appears in the default failure path
- partial success remains clearly repair-accounted inline
- partial success no longer compresses committed / failed path accounting behind omission prose
- failure-time patch source persistence exists when inline patches fail and no preexisting patch file is available
- warning-free success output remains stable
- tests cover both source-span-rich and source-span-poor cases
- no diagnostic rendering step invents fake precision

## 15. Relationship to scheduling

This document is intentionally not the implementation schedule.

Scheduling belongs in a separate temporary execution plan so that:

- the design remains stable even if sequencing changes
- implementers can revise order without rewriting the specification
- reviewers can judge whether the implementation still matches the intended
  diagnostics model
