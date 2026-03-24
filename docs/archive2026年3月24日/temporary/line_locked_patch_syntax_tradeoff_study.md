# Line-Locked `apply_patch` Syntax Trade-off Study

## Status

- Recorded on 2026-03-23.
- Scope: syntax candidate analysis for the future line-locked `apply_patch` extension.
- Method: QOC-oriented comparison with explicit rejection notes.

## Exclusions

This document does not:

- serve as the final normative grammar
- replace the design record
- replace the black-box evaluation program

## Question

How should a line-locked `apply_patch` extension express stronger target locking
while preserving the practical strengths of patch-shaped editing?

## Criteria

The following criteria are used consistently across candidates:

1. authoring ergonomics
2. parser complexity
3. ambiguity resistance
4. diagnostics quality
5. token cost
6. host teachability
7. cross-model intuitiveness
8. preservation of patch muscle memory

## Option A: Maximal Numbered Old-Side Evidence Pattern

### Shape

```text
*** Begin Patch
*** Update File: src/app.py
@@
 120 | def handler():
-121 |     value = 1
-122 |     return value
+    value = 2
+    return value
*** End Patch
```

### Strengths

- strongest explicit old-side lock
- highest diagnostic precision
- good for file-start insertion and highly repetitive local regions

### Weaknesses

- higher token cost
- visually heavier
- weaker continuity with ordinary patch muscle memory

### Judgment

- strongest evidence pattern
- not the preferred default authoring form

## Option B: Concise Single-Anchor Form

### Shape

```text
*** Begin Patch
*** Update File: src/app.py
@@ 120 | def handler():
-    value = 1
-    return value
+    value = 2
+    return value
*** End Patch
```

### Strengths

- strongest balance between familiarity and stronger lock
- low added syntax burden
- keeps ordinary diff body semantics intact
- gives `@@` one precise role

### Weaknesses

- anchor success does not mathematically eliminate all later ambiguity
- weaker than the maximal numbered pattern in highly repetitive post-anchor regions
- less natural for some file-start insertion cases

### Judgment

- preferred prompt-facing authoring form
- not sufficient as the only parser-supported form

## Option C: Preferred Concise Form Plus Stronger Numbered Evidence

### Shape

- concise single-anchor form is taught first
- denser numbered old-side evidence remains available when ambiguity persists

### Strengths

- preserves ergonomic default path
- preserves expressive coverage for long-tail cases
- supports host escalation without inventing a new mechanism
- keeps the primary escalation path simple: switch to the maximal numbered pattern when
  concise anchoring is not enough
- keeps parser support and prompt preference aligned but not collapsed

### Weaknesses

- slightly more complex to explain than a single-form language
- requires disciplined prompt teaching and diagnostics wording

### Judgment

- current best direction

## Option D: Stacked Multiple `@@` Anchor Hierarchy

### Shape

```text
@@ class A
@@ def run(self):
@@ def inner():
```

### Strengths

- superficially suggests stronger structure

### Weaknesses

- not supported by the current parser family
- weak cross-model intuitiveness
- high documentation and diagnostics complexity
- low evidence of irreplaceable practical benefit

### Judgment

- deprioritized
- not a current investment target

## Option E: Separate `CTX` Mechanism

### Shape

- add a distinct grammar object whose role is to re-ignite attention mid-span

### Strengths

- attempts to name the need for mid-span reinforcement explicitly

### Weaknesses

- treats a means as if it were the primary object
- duplicates the function that chunk decomposition and stronger numbered-evidence patterns can
  already cover
- risks adding a new object category without enough parser/runtime payoff

### Judgment

- rejected as a first-class syntax object
- absorbed into chunk decomposition strategy and host escalation guidance instead

## Mapping Precision Notes

To avoid overclaiming, the candidates should be interpreted with the following
precision levels:

- Option A: strongest current evidence pattern
- Option B: strongest ergonomic candidate
- Option C: current architectural direction
- Option D: consciously deprioritized hierarchy path
- Option E: consciously rejected standalone mechanism

## Recommendation

The current recommendation is Option C:

- teach Option B first
- support Option A as the stronger evidence pattern
- do not pursue Option D or Option E as primary design paths

This recommendation best satisfies the combined objective:

- stronger lock than context-only patching
- preservation of patch-shaped editing
- low prompt burden
- clean escalation path for difficult cases

## Implication for Formal Grammar Work

The future grammar draft should therefore be structured around:

1. a concise anchor chunk form
2. a stronger numbered-evidence chunk form
3. a shared file-level patch envelope
4. explicit host guidance for when to switch forms

That structure keeps the syntax family coherent without pretending that one form
must do every job equally well.
