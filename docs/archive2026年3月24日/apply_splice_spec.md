# `apply_splice` Detailed Design

## Status

- Stable product/design contract for `apply_splice`
- Defines the tool's object boundary, authored surface, and locked semantic
  decisions before implementation
- Scheduling, extraction sequencing, and other implementation-prep mechanics live
  in separate temporary planning artifacts

## 1. Problem statement

Large-model editing workflows often pay a high cost when the model must restate
large blocks of already-existing text merely to move or copy them elsewhere.

This cost appears in several forms:

- repeated output token cost
- duplicated context pressure
- avoidable re-expression drift when the content already exists verbatim

`apply_patch` is the correct tool for structural editing with new authored text.
It is not optimized for the narrower case where the model only wants to:

- select an existing block
- move or copy it
- place it at another existing location

This document proposes a distinct tool for that narrower case.

## 2. Tool identity

The tool name is:

- `apply_splice`

Why `splice`:

- it implies cut/copy/paste-like structural transfer
- it is narrower than `patch`
- it does not imply free-form text generation
- it naturally fits source-to-target block movement semantics

## 3. Core product idea

`apply_splice` is a structural operation tool for existing text spans.

It allows the model to express:

- which existing text block to select
- whether that block is copied, moved, or deleted
- if transferred, how it is placed relative to the target

It does not allow the model to author new text inline.

In short:

- `apply_patch` = author or rewrite text
- `apply_splice` = relocate, duplicate, or delete existing text

## 4. Non-goals

`apply_splice` does not:

- become an alternate free-form patch language
- allow inline creation of new text
- allow partial rewriting of the selected source text during the splice itself
- rely on JSON coordinate payloads as the primary authored surface
- become a general refactoring engine

## 5. Primary design principle

The tool is narrow, composable, and unsurprising.

It feels like:

- a structural copy/move instruction language

and not like:

- a second general editing DSL with a different accent

### Decision D1.5: selections use double-lock validation

Selections are validated by two independent signals:

- absolute 1-indexed line numbers
- visible content on the shown numbered lines

This double-lock is one of the main reasons `apply_splice` can stay compact
without becoming ambiguous.

## 6. Action model

The action basis has two families.

### 6.1 Transfer family

The transfer family is factored into two orthogonal dimensions.

Source-side modes:

- `Copy`
- `Move`

Target-side modes:

- `Append`
- `Insert Before`
- `Insert After`
- `Replace`

These dimensions yield exactly eight transfer actions.

### 6.2 Removal family

One source-only primitive also exists:

- `Delete Span`

`Delete Span` removes a selected existing span without target relocation and
without inline authored replacement.

Taken together, the current action basis is:

- eight transfer actions from the `Copy/Move × Append/Insert Before/Insert After/Replace` matrix
- one removal primitive: `Delete Span`

## 7. Canonical authored shape

The authored surface is envelope-shaped and patch-like, but it does not reuse
patch hunk grammar literally.

Canonical example:

```text
*** Begin Splice
*** Copy From File: source.py
@@
120 | def build_context(...)
... source lines omitted ...
128 |     "mode": "strict",
*** Append To File: target.py
*** End Splice
```

For replacement:

```text
*** Begin Splice
*** Move From File: source.py
@@
120 | def build_context(...)
... source lines omitted ...
128 |     "mode": "strict",
*** Replace In File: target.py
@@
45 | old block start
... target lines omitted ...
52 | old block end
*** End Splice
```

This keeps the source and target semantics explicit and separate.

For deletion:

```text
*** Begin Splice
*** Delete Span From File: source.py
@@
120 | def obsolete_helper(...)
... source lines omitted ...
128 |     return old_value
*** End Splice
```

This keeps span deletion explicit rather than smuggling it through authored
replacement syntax.

## 8. Selection contract

The source and target selections must be stable and low-ambiguity.

### 8.1 Required properties

- selections are line-oriented
- selections carry absolute 1-indexed line numbers
- selections permit vertical omission only
- selections do not permit horizontal truncation

The contract is double-lock validation:

- line numbers lock the intended absolute span
- visible content locks the semantic identity of the boundary lines

### 8.2 Why this contract is strong

The selection is validated twice:

- by absolute line numbers
- by the visible content shown on those numbered lines

This provides a strong lock against ambiguity while still letting the model write
a compact excerpt surface.

### 8.3 Meaning of omission inside a selection

Inside a source or target selection, the authoritative omission markers are:

- source selection: `... source lines omitted ...`
- target selection: `... target lines omitted ...`

An omission marker means:

- the selection remains contiguous
- omitted lines between the neighboring numbered lines are part of the selected
  span

This is not a sparse sample.
It is a compact way to denote a contiguous range.
Bare `...` from sampled inspection output is not canonical in `apply_splice`.

## 9. Why horizontal truncation must be forbidden here

The `read_file` sampled view can tolerate horizontal truncation because it is a
confidence-oriented inspection tool.

`apply_splice` cannot.

Reasoning:

- a splice selection must name exact existing text
- horizontal truncation would weaken the content anchor
- the tool must preserve deterministic source/target matching behavior

Therefore:

- line-numbered excerpts with vertical omission are allowed
- `...[N chars omitted]` is not allowed inside splice selections

## 10. Operational semantics

### 10.1 Copy + Append

- read the selected source range
- append it to the end of the target file
- source remains unchanged

### 10.2 Move + Append

- read the selected source range
- append it to the end of the target file
- delete the selected source range

### 10.3 Copy / Move + Insert Before / After

- resolve the target anchor selection
- insert the source block before or after that contiguous target range
- remove the source range only for `Move`

### 10.4 Copy / Move + Replace

- resolve the source range
- resolve the target range
- replace the target range with the source block
- remove the source range only for `Move`

### 10.5 Delete Span

- resolve the source range
- remove the selected source range
- apply no target-side insertion or replacement

## 11. Atomicity requirements

`apply_splice` inherits the same seriousness about commit behavior that
`apply_patch` now has.

Design requirement:

- each splice action is a connected mutation unit across all touched files
- a `Move` must not leave behind a half-completed state where the destination was
  written but the source was not removed, or vice versa

In practice this means the tool reuses or mirrors the existing file-group
atomicity discipline rather than inventing a looser model.

## 12. Multi-action programs

The tool allows multiple splice actions inside one envelope, similar to a
patch program containing multiple operations.

This is useful for:

- extracting several helpers in one pass
- relocating a sequence of existing blocks
- combining one copy and one replace in the same logical edit program

However, each action is still modeled as an explicit source-to-target
instruction, not as an implicit batch transform.

## 13. Diagnostics philosophy

`apply_splice` does not invent a totally different diagnostics language.

It aligns with the existing DocuTouch patch tooling philosophy:

- stable codes
- compact repair-oriented output
- truthful blame locations
- inline failure messages as the primary repair surface, with optional failed-program-source persistence when the original splice program was not already file-backed
- preserved repair accounting when partial success exists
- host-owned audit trails rather than audit-shaped tool-managed sidecars

Expected high-frequency failure classes:

- source file drift
- target file drift
- line-number/content mismatch
- source/target overlap in illegal ways
- write failures during connected multi-file updates

## 14. Relationship to `apply_patch`

`apply_splice` is not folded into `apply_patch`.

Reasoning:

- the capability boundary is different
- the authoring model is different
- the success condition is different
- the tool should remain visibly constrained to existing text movement

At the same time, the implementation reuses as much runtime machinery as
possible from the existing stack:

- parsing helpers where appropriate
- path handling
- file-group atomicity
- diagnostics style
- repair-first diagnostics contract, stable failed-program-source references when needed, and a host-owned audit boundary

This is shared implementation, not shared product identity.

### Clarification on the vendored baseline

The project does not need to treat the vendored OpenAI patch code as untouchable.

The correct boundary is:

- it is acceptable to modify vendored code in order to extract generic shared
  correctness substrate
- it is not desirable to collapse `apply_splice` into `apply_patch`'s public
  grammar or product identity

In other words:

- code ownership may be shared
- product identity must remain separate

## 15. Complete action basis

The action basis is:

- `Delete Span From File`
- `Copy From File` + `Append To File`
- `Move From File` + `Append To File`
- `Copy From File` + `Insert Before In File`
- `Copy From File` + `Insert After In File`
- `Move From File` + `Insert Before In File`
- `Move From File` + `Insert After In File`
- `Copy From File` + `Replace In File`
- `Move From File` + `Replace In File`

These actions are not a provisional version slice.
They are the current complete action basis: eight transfer actions plus one
source-only delete primitive.

## 16. Current exclusions implied by tool boundary

The following stay out of `apply_splice` unless the product boundary itself is intentionally changed:

- inline editing of the moved/copied block
- multi-range source aggregation in one action
- fuzzy or semantic matching beyond numbered excerpt validation
- auto-generated destination scaffolding
- swap/merge/dedupe macro-operations

If those ever become desirable, they must be justified as new product work,
not smuggled in under a narrow splice tool.

## 17. Design-lock decisions before implementation

The following decisions are locked for the next implementation-prep phase.

### Decision D6: same-file source and target selections resolve against the original snapshot

Reasoning:

- it keeps selection meaning stable
- it avoids hidden dependence on intermediate mutation order
- it makes diagnostics easier to explain

Operational consequence:

- source and target ranges are both resolved against the same pre-mutation file
  snapshot
- runtime may then translate target offsets after source removal as an execution
  detail

### Decision D7: overlapping same-file source and target ranges are illegal

Reasoning:

- overlap semantics are high-risk and easy to misunderstand
- rejecting overlap keeps the tool narrow and easier to account for inline

Operational consequence:

- if source and target ranges overlap in the same file for `Insert Before`,
  `Insert After`, or `Replace`, the tool should fail with a stable overlap-class
  diagnostic rather than trying to guess intent

### Decision D8: target existence rules stay narrow

Rules:

- `Append To File` may create a missing destination file
- `Insert Before In File`, `Insert After In File`, and `Replace In File` require
  the target file and target selection to already exist

Reasoning:

- append-to-new-file is a natural structural transfer case
- insert/replace are anchor-driven and should not fabricate missing context

### Decision D9: source text is preserved verbatim, including newline bytes

Reasoning:

- the tool is reference-preserving by design
- silent newline normalization would weaken that identity

Operational consequence:

- the selected source bytes should be transferred as-is
- mixed newline styles may remain possible and can later become a warning topic,
  but they are not silently rewritten

### Decision D10: diagnostics use a stable blame hierarchy

Preferred blame order:

1. source selection location for source-selection mismatch
2. target selection location for target-selection mismatch
3. action header location for semantic/ordering/overlap errors
4. target path or destination anchor for write-stage failures when no better
   authored location exists

Reasoning:

- the caller should learn the minimum next repair move
- blame should follow the most truthful authored location first
- runtime-only failures should not fabricate stronger authored precision than is
  actually available

## 18. Acceptance criteria

`apply_splice` counts as successful only if:

- the source/target contract is immediately understandable
- the tool never allows the model to invent new text inline
- selections remain low-ambiguity because of line-number and content validation
- move actions remain atomic across source and destination
- diagnostics stay compact and repair-oriented
- the action surface remains small enough to stay comfortable to author

## 19. Relationship to scheduling

This document is intentionally not the implementation schedule.

Scheduling belongs in a separate temporary plan so that design stability and
execution order remain decoupled.
