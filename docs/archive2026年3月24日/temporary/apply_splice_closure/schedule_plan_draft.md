# `apply_splice` Implementation Schedule Draft

## Status

- Draft schedule from the current closure-complete / code-not-started state to v1 done
- Scope: implement the documented `apply_splice` promises and close all required tests
- Effort unit: idealized engineering days for a focused engineer after entry criteria are met
- Review latency is intentionally modeled as gates, not hidden inside effort numbers
- In this document, `v1` means the first implementation/release closure of the
  already-closed product contract; it does not mean the current action basis is a
  negotiable scope menu

## 1. Delivery Objective

The delivery target is not "some splice code exists." The delivery target is:

1. `apply_splice` ships as a DocuTouch-owned tool distinct from `apply_patch`.
2. The public contract matches the stable documentation and promoted closure decisions.
3. The full current action basis is implemented:
   eight transfer actions from `Copy/Move x Append/Insert Before/Insert After/Replace`
   plus the source-only `Delete Span` primitive.
4. Selection semantics are deterministic and test-backed:
   absolute numbered lines, visible-content validation, explicit omission handling,
   and no horizontal truncation.
5. Same-file semantics are correct and fully accounted for:
   original-snapshot interpretation, illegal overlap rejection, translated offsets
   only after denotation.
6. Byte transfer is reference-preserving:
   source bytes, separators, and EOF-without-newline behavior are preserved.
7. Execution uses connected-unit atomicity with truthful partial-success behavior
   across disjoint units.
8. Diagnostics, success summaries, MCP output, and CLI output are all
   stable, repair-oriented, and aligned with DocuTouch family conventions.
9. The Rust workspace tests pass at all affected layers.

## 2. Source-of-Truth Baseline

The schedule should treat the following artifacts as the current pre-coding input set.

| Artifact | Current role | What must happen before implementation is considered safe |
| --- | --- | --- |
| `docs/apply_splice_spec.md` | stable product/design contract | already carries the closed product-boundary decisions; remaining promotion work is implementation-facing detail |
| `docs/temporary/apply_splice_closure/formal_semantics_draft.md` | temporary implementation-driving semantics | promote canonical grammar, selection relation, byte-span semantics, and same-file rules into stable form |
| `docs/temporary/apply_splice_closure/architecture_diagnostics_test_draft.md` | temporary architecture/diagnostics/QA closure | promote shared-boundary, diagnostics family, and recursive QA obligations into stable form |
| `docs/temporary/apply_splice_closure/stage_summary.md` | closure decision register | convert closed decisions into stable contract text; keep only historical context temporary |
| `docs/temporary/apply_splice_technical_investigation.md` | architecture recommendation | use as rationale for extraction and non-reuse boundaries |
| `docs/maintainer_guide.md` | long-term maintenance rules | enforce doc-sync and three-layer test discipline during implementation |

## 3. Current Starting State

The repository is not starting from zero, but it is also not in implementation.

| Area | Observed state now | Scheduling consequence |
| --- | --- | --- |
| Product direction | stable enough | no new discovery phase is justified |
| Canonical grammar text | closure draft exists, stable doc still incomplete | stable-promotion phase is mandatory |
| Architecture boundary | recommended and fairly concrete | extraction-plan phase is mandatory |
| Diagnostics family | drafted, not fully locked in stable docs | diagnostics lock must precede outward surfaces |
| Test model | recursive model drafted, not implemented | test work must be planned from inner layers outward |
| Code implementation | no `apply_splice` runtime/parser/wiring currently present | schedule must include full implementation path |
| User-facing tool docs | not present | shipping phase must include new tool docs |

## 4. Completion Standard

`apply_splice` should only be declared done when all of the following are true:

1. No normative v1 rule depends on temporary closure notes alone.
2. Stable docs and executable behavior agree on grammar, semantics, diagnostics,
   and outcome reporting.
3. The shared mutation substrate boundary is extracted or otherwise landed in the
   approved shape without leaking splice-specific semantics downward.
4. The full current action basis is implemented and covered by positive and negative tests.
5. Partial success and connected-unit rollback behavior are validated under tests.
6. MCP and CLI surfaces both expose the tool and agree on observable behavior.
7. User-facing documentation exists and does not promise behavior the runtime does
   not implement.
8. Residual non-goals remain explicit and no latent "v1.5" scope has been smuggled
   into the implementation.

## 5. Phase Graph And Critical Dependencies

The critical path is intentionally linear at the contract and boundary stages.
Coding may overlap later only after the relevant review gate passes.

| Phase | Name | Depends on | Can overlap with | Blocks |
| --- | --- | --- | --- | --- |
| 1 | Stable Contract Promotion | none | none | every coding phase |
| 2 | Architecture Boundary Lock | Phase 1 | none | Phases 3-5 |
| 3 | Shared Substrate Extraction | Phase 2 | late Phase 4 after codec/API lock only | Phases 5-7 |
| 4 | Parser And Selection Engine | Phase 1 and Phase 2 | tail of Phase 3 once excerpt codec contract is frozen | Phases 5-7 |
| 5 | Runtime, Diagnostics, Presentation | Phases 3 and 4 | late Phase 6 on documentation drafting only | Phases 6-7 |
| 6 | Server Wiring And User Docs | Phase 5 | none in a closure-critical sense | Phase 7 |
| 7 | Verification, Hardening, Release Closure | Phase 6 | none | final declaration |

Expected critical-path effort band: 21-34 idealized engineering days.

## 6. Phase Plan

## Phase 1. Stable Contract Promotion

**Purpose**

Convert the closure outputs from "good temporary material" into the authoritative
stable contract that code is allowed to implement.

**Why this phase is necessary**

The closure review already concluded that the next valuable step is controlled
promotion, not another discovery loop. Coding directly against temporary drafts
would make avoidable drift likely.

**Dependencies**

- none

**Estimated effort**

- 2-4 idealized engineering days

**Work packages**

1. Decide the long-term home of the canonical grammar and formal semantics.
2. Promote the authoritative omission-token wording into stable documentation.
3. Promote the formal selection validation relation.
4. Promote same-file original-snapshot interpretation and overlap illegality.
5. Promote byte-span and newline-fidelity wording.
6. Promote the minimum diagnostics-family contract and blame hierarchy.
7. Promote the recursive QA model and minimum scenario matrix into a stable
   implementation-facing document or section.
8. Create a requirement-to-test acceptance matrix so each promise has an intended
   implementation layer and test owner.

**Entry criteria**

1. Closure artifacts have been read and reconciled against `docs/apply_splice_spec.md`.
2. Any remaining naming-only disagreements are explicitly listed.
3. No one is still arguing for reopening product identity or action-surface scope.

**Exit criteria**

1. Stable docs contain the canonical v1 grammar and all normative semantics needed
   to begin coding.
2. No normative behavior remains documented only in temporary closure notes.
3. A requirement matrix exists that maps each promise to code layer, test layer,
   and review gate.
4. Temporary closure docs become supporting rationale rather than sole authority.

**Risk checkpoint**

Stop if any of the following is still unresolved after the drafting pass:

- exact omission-token spelling
- diagnostics-family names or boundaries
- host-audit boundary
- whether same-file translation rules are sufficiently explicit for implementation

If unresolved, do not start code. Resolve the contract first.

**Review gate: RG1 Contract Gate**

Required evidence:

1. Stable-doc diff reviewed and approved.
2. Decision register updated to show what moved from temporary to stable.
3. Implementation team confirms there is no remaining blocker that would force
   speculative coding.

## Phase 2. Architecture Boundary Lock

**Purpose**

Freeze the implementation boundary between shared correctness substrate and
splice-owned semantics before extraction or parser/runtime work begins.

**Why this phase is necessary**

This is the phase that prevents two opposite failure modes:

1. re-implementing hard-won patch correctness in parallel
2. polluting a shared layer with splice-specific semantics

**Dependencies**

- Phase 1 complete

**Estimated effort**

- 1-3 idealized engineering days

**Work packages**

1. Name the shared layer location and public API boundary.
2. Freeze the reusable set:
   path identity, connected-unit grouping, staged commit/rollback,
   affected-path summarization, generic rendering helpers, numbered-excerpt codec.
3. Freeze the explicit non-reuse set:
   patch grammar, diff/hunk matching, patch-specific sidecar identity,
   sampled-read horizontal truncation semantics.
4. Decide whether numbered-excerpt parsing lives in a tiny shared codec module or
   a new low-level crate.
5. Define the splice-owned module map:
   parser, selection, runtime, presentation, server wiring.
6. Define the test ownership split across `codex-apply-patch`, `docutouch-core`,
   and `docutouch-server`.

**Entry criteria**

1. RG1 passed.
2. The promoted contract names the semantics that the architecture must support.
3. The maintainers agree that `apply_splice` stays a separate product surface.

**Exit criteria**

1. A concrete module/API map exists and is approved.
2. The shared layer has an explicit "must not know" rule for splice semantics.
3. The extraction backlog is small, ordered, and reviewable.
4. Test ownership per layer is documented before any code lands.

**Risk checkpoint**

If any proposed shared helper needs to know source-vs-target semantics, overlap
legality, omission-token meaning, or newline-preservation rules, the boundary is
too high-level and must be reduced before coding continues.

**Review gate: RG2 Boundary Gate**

Required evidence:

1. Approved boundary note or design diff.
2. Named module map and API seams.
3. Agreement on where regression responsibility lives after extraction.

## Phase 3. Shared Substrate Extraction

**Purpose**

Extract or isolate the lower correctness substrate that splice needs, while keeping
existing patch behavior intact.

**Why this phase is necessary**

The project already owns hard correctness problems in the patch stack. Re-solving
them inside splice would duplicate risk exactly where the codebase is already most
fragile: path identity, commit units, rollback, and affected-path accounting.

**Dependencies**

- RG2 passed

**Estimated effort**

- 4-6 idealized engineering days

**Work packages**

1. Extract path normalization / path identity helpers into the approved lower layer.
2. Extract connected-unit grouping so multi-file atomicity is not patch-private.
3. Extract staged commit / rollback planning in a form that splice can consume.
4. Extract or generalize affected-path summarization for family-compatible `A/M/D`.
5. Extract generic diagnostic-rendering helpers only where genuinely cross-tool.
6. Land regression tests proving existing `apply_patch` behavior still holds.
7. Update vendored-baseline notes if the extraction creates a new documented local
   divergence or formally records an existing one.

**Entry criteria**

1. RG2 passed.
2. A concrete extraction list exists and excludes splice semantics.
3. Patch maintainers agree on regression expectations before refactoring starts.

**Exit criteria**

1. Shared correctness helpers are callable from non-patch code.
2. Existing patch tests remain green with no silent behavior drift.
3. The extracted layer does not expose patch grammar vocabulary or patch-sidecar
   identity as its public API.
4. The shared layer is small enough that a reviewer can understand it as a lower
   substrate, not a second product surface.

**Risk checkpoint**

The primary risk is regression in current patch behavior. If patch tests fail or
behavior becomes ambiguous, stop extraction and repair the substrate before any
splice runtime work proceeds.

**Review gate: RG3 Substrate Gate**

Required evidence:

1. Extraction diff reviewed by maintainers of the patch stack.
2. Existing patch tests pass.
3. Reviewers agree the lower layer is still generic correctness substrate.

## Phase 4. Parser And Selection Engine

**Purpose**

Implement the splice-owned authored surface and deterministic selection resolver.

**Why this phase is necessary**

`apply_splice` is not a diff application problem. Its highest semantic risk is not
write I/O; it is getting denotation wrong. Parser and selection work therefore need
their own phase rather than being buried inside runtime coding.

**Dependencies**

- RG1 passed
- RG2 passed
- any shared numbered-excerpt codec contract frozen

**Estimated effort**

- 4-6 idealized engineering days

**Work packages**

1. Implement envelope parsing for `*** Begin Splice` / `*** End Splice`.
2. Implement source and target header parsing with source-map locations.
3. Implement selection-block parsing for numbered excerpt lines.
4. Enforce authored-shape rules for append versus anchored target modes.
5. Enforce omission-token placement and side-specific token rules.
6. Reject horizontal truncation markers and malformed delimiters.
7. Resolve contiguous source and target intervals via the double-lock relation.
8. Surface semantic overlap inputs needed by same-file validation.
9. Add dedicated unit tests for grammar validity, invalid-shape rejection,
   excerpt decoding, and selection denotation.

**Entry criteria**

1. RG1 and RG2 passed.
2. Canonical grammar and omission-token wording are stable.
3. The excerpt codec contract is not expected to change underneath the parser.

**Exit criteria**

1. The parser yields a semantic action representation independent of filesystem I/O.
2. Every grammar rule and invalid authored shape in the stable contract has at
   least one direct test.
3. Selection resolution failures map cleanly to source or target authored spans.
4. No runtime code is compensating for parser ambiguity.

**Risk checkpoint**

If the parser needs recovery heuristics or fuzzy matching to interpret ordinary
examples, the authored surface is still underspecified. Stop and correct the spec
or parser model instead of hiding the problem in runtime code.

**Review gate: RG4 Semantic Input Gate**

Required evidence:

1. Parser and selection tests cover the full validity matrix.
2. Reviewers can trace each stable grammar rule to a concrete parse/validation path.
3. Failure locations are truthful at the authored-surface layer.

## Phase 5. Runtime, Diagnostics, And Presentation

**Purpose**

Implement the actual transfer engine, same-file rules, diagnostics, success
summaries, and partial-success behavior on top of the extracted substrate.

**Why this phase is necessary**

This phase is where most of the product promise becomes real. It is also the phase
with the highest correctness risk because the logic now spans source denotation,
target denotation, byte copying, move removal, commit-unit planning, and user-visible
failure behavior.

**Dependencies**

- RG3 passed
- RG4 passed

**Estimated effort**

- 5-8 idealized engineering days

**Work packages**

1. Implement snapshot capture for all touched paths per commit unit.
2. Implement target-point and target-range derivation for append / insert before /
   insert after / replace.
3. Implement source-byte transfer preserving exact separators and EOF behavior.
4. Implement copy semantics across same-file and cross-file cases.
5. Implement move semantics, including source removal and same-file offset
   translation after denotation.
6. Enforce missing-target policy:
   append may create missing file, insert/replace may not.
7. Enforce illegal same-file overlap rejection before writes begin.
8. Implement connected-unit atomic commit and rollback using the shared substrate.
9. Implement partial-success reporting across disjoint units.
10. Implement stable diagnostics family, blame hierarchy, and repair-first
    failure contract.
11. Implement family-compatible success summaries, including destination-side `M`
    normalization for move-shaped success.
12. Add runtime and presentation tests for the full action basis, newline fidelity,
    overlap errors, write errors, partial success, and summary rendering.

**Entry criteria**

1. RG3 and RG4 passed.
2. There is no remaining ambiguity about same-file semantics or outcome reporting.
3. The shared substrate APIs are stable enough to build against.

**Exit criteria**

1. The full current action basis executes according to the stable contract.
2. Same-file behavior is correct for legal and illegal cases.
3. Runtime failures and partial failures produce stable, truthful diagnostics.
4. Success summaries are aligned with the DocuTouch family baseline.
5. Inner-layer tests prove the action matrix and core edge cases.

**Risk checkpoint**

The highest-risk failure modes are:

- same-file move offset mistakes
- source-byte reconstruction instead of verbatim transfer
- incorrect rollback or partial-success accounting
- summary drift for move-shaped outcomes

Any one of these is release-blocking. Do not defer them to outer smoke tests.

**Review gate: RG5 Runtime Gate**

Required evidence:

1. Action-matrix tests pass.
2. Same-file and newline-fidelity tests pass.
3. Reviewers can map each diagnostic code to a distinct semantic failure class.
4. Partial-success output is repair-accounted inline and not over-compressed.

## Phase 6. Server Wiring And User Documentation

**Purpose**

Expose the tool through the supported surfaces and publish the contract that users
and agents will actually read.

**Why this phase is necessary**

The tool is not done when core code exists. It is done when the agent-facing
surface matches the runtime and the repository no longer relies on temporary notes
to explain how to use the tool.

**Dependencies**

- RG5 passed

**Estimated effort**

- 2-3 idealized engineering days

**Work packages**

1. Register the tool in `docutouch-server` and wire request/response handling.
2. Ensure MCP and CLI share the same semantics and message structure.
3. Add `tool_docs/apply_splice.md` with examples that match the implemented rules.
4. Update stable docs and top-level navigation docs where the new tool must now be
   discoverable.
5. Add server and CLI smoke tests for both success and failure.

**Entry criteria**

1. RG5 passed.
2. Runtime outputs are stable enough that user docs will not immediately drift.

**Exit criteria**

1. The tool is invocable through every intended surface.
2. User-facing docs exist and match implemented behavior.
3. Observable MCP and CLI behavior is covered by smoke tests.
4. No temporary closure note is required for ordinary usage understanding.

**Risk checkpoint**

Transport drift is the main risk here. If CLI, MCP, and tool docs do not agree on
headlines, codes, or inline-diagnostics / host-audit semantics, the tool is not
ready for verification.

**Review gate: RG6 Surface Gate**

Required evidence:

1. Tool registration and smoke tests reviewed.
2. Tool docs reviewed against actual output fixtures.
3. No user-visible example contradicts runtime behavior.

## Phase 7. Verification, Hardening, And Release Closure

**Purpose**

Close the final confidence gap between "feature implemented" and
"documentation promises fully implemented and tests passing."

**Why this phase is necessary**

The codebase already has a strong maintainer rule: boundary fixes require tests,
and user-visible contract changes require outer-surface verification. This phase is
where those rules are enforced as a release decision rather than as aspirations.

**Dependencies**

- RG6 passed

**Estimated effort**

- 3-5 idealized engineering days

**Work packages**

1. Complete the minimum scenario matrix across all layers.
2. Run the full affected Rust test suites and fix remaining regressions.
3. Perform doc drift review:
   stable spec, tool docs, examples, README references, and any recorded baseline
   notes must agree with the implementation.
4. Verify no leftover temporary wording is still normative.
5. Confirm non-goals remain explicit and no unreviewed scope growth landed.
6. Prepare merge package with review notes, residual-risk statement, and release
   checklist evidence.

**Entry criteria**

1. RG6 passed.
2. The full scenario matrix is defined, not merely implied.
3. There is a named owner for any remaining flaky or slow test work.

**Exit criteria**

1. All required tests pass.
2. All documentation promises are implemented or explicitly narrowed in the same
   workstream before merge.
3. Residual known risks are either fixed or consciously accepted as non-blocking.
4. The project can truthfully say the tool is done without pointing to future work
   for core contract behavior.

**Risk checkpoint**

If the only remaining mismatches are documentation or examples, they are still
release blockers. For this tool, docs are part of the contract, not optional polish.

**Review gate: RG7 Release Gate**

Required evidence:

1. Required test suites green.
2. Stable docs, tool docs, and examples reviewed against the final runtime.
3. Merge reviewers agree the tool meets the published v1 contract.

## 7. Cross-Phase Control Rules

## Before coding may start

All of the following must be true before any parser/runtime implementation begins:

1. RG1 passed.
2. Grammar, omission tokens, same-file rules, newline policy, diagnostics family,
   and blame hierarchy are stable enough to code against.
3. The requirement-to-test matrix exists.
4. The architecture boundary has at least a provisional approved shape.
5. No one is planning to rely on temporary closure prose as the only source of
   truth for a normative rule.

## During coding must remain true

These are continuous engineering-management invariants, not end-of-project wishes:

1. Every semantic rule gets tests at the cheapest truthful layer first.
2. Any contract change made during implementation updates the stable docs in the
   same workstream.
3. Shared substrate code remains splice-agnostic.
4. Patch regressions discovered during extraction are treated as priority work,
   not as acceptable collateral damage.
5. No new scope enters v1 without an explicit contract review.
6. Success summaries and diagnostics are reviewed as contract surfaces, not just UI.

## Before merge must be true

1. RG7 is ready to pass with evidence, not with promises.
2. The full action basis is fully implemented and covered.
3. The minimum scenario matrix is complete.
4. MCP and CLI parity tests pass.
5. Stable docs and user-facing tool docs match actual runtime behavior.
6. Any divergence from patch baseline or upstream lineage caused by shared-layer
   work is documented.

## Before declaring the tool done must be true

1. All release-blocking defects from verification are closed.
2. There is no known gap where docs promise behavior that only partially exists.
3. The repository contains the long-term artifacts needed to maintain the tool:
   stable spec, tool docs, tests, and any baseline notes required by the change.
4. The temporary closure directory can be treated as historical context rather than
   active operational dependency.

## 8. Review Gates Summary

| Gate | Decision question | Earliest phase | If gate fails |
| --- | --- | --- | --- |
| RG1 Contract Gate | Is there a stable normative contract to implement? | 1 | stop coding and resolve docs |
| RG2 Boundary Gate | Is the shared-vs-owned split explicit and narrow enough? | 2 | stop extraction and redesign the seam |
| RG3 Substrate Gate | Can splice reuse the lower layer without patch regression? | 3 | repair the substrate before proceeding |
| RG4 Semantic Input Gate | Is authored-surface parsing and denotation deterministic? | 4 | stop runtime work and fix parser/selection semantics |
| RG5 Runtime Gate | Do execution, diagnostics, and summaries match the contract? | 5 | do not expose the tool yet |
| RG6 Surface Gate | Do MCP, CLI, and docs match the runtime? | 6 | do not begin release verification |
| RG7 Release Gate | Are the documentation promises actually implemented and passing? | 7 | no merge, no done declaration |

## 9. Recommended Execution Notes

1. Do not combine Phase 1 with runtime coding. That would collapse the distinction
   between contract definition and implementation and make later review weaker.
2. Do not postpone the shared substrate work until after a private splice runtime
   exists. That path would maximize rework.
3. Do not rely on end-to-end tests alone. The closure work explicitly requires a
   recursive model, and the maintainer guide already warns against pushing all
   verification outward.
4. Expect the highest review intensity at Phases 3 and 5. Those phases contain the
   main correctness risk, not just the most code.
5. Treat documentation examples as test-adjacent artifacts. Every example used in
   tool docs should be validated against current runtime behavior before merge.

## 10. Compact Milestone Table

| Milestone | Outcome | Depends on | Effort band |
| --- | --- | --- | --- |
| M1 | Stable `apply_splice` contract promoted from closure artifacts | none | 2-4 days |
| M2 | Shared-vs-owned architecture boundary approved | M1 | 1-3 days |
| M3 | Shared mutation substrate extracted without patch regression | M2 | 4-6 days |
| M4 | Parser and selection engine implemented and test-backed | M1, M2 | 4-6 days |
| M5 | Runtime, diagnostics, and presentation implemented for all v1 actions | M3, M4 | 5-8 days |
| M6 | MCP/CLI wiring and user docs landed | M5 | 2-3 days |
| M7 | Full verification complete; docs promises and tests closed | M6 | 3-5 days |

Critical-path total: 21-34 idealized engineering days.
