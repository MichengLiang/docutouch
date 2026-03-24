# Line-Locked `apply_patch` Extension Design Record

## Status

- Recorded on 2026-03-23.
- Scope: design record for the future line-locked `apply_patch` grammar extension.
- Authority level: temporary product/design record; not yet a stable public spec.

## Exclusions

This document does not:

- replace the confirmed defect register
- serve as the syntax trade-off matrix itself
- serve as the black-box evaluation program
- define the current implementation schedule

Those responsibilities are split into separate artifacts.

## Purpose

This document captures the current direction behind a line-locked grammar
extension for `apply_patch` that uses stronger target locking than the existing
context-matching forms.

Its job is to preserve the reasoning that would otherwise disappear into chat
history, especially around:

- why stronger locking is being added to `apply_patch`
- how it should relate to ordinary `apply_patch`
- how it should relate to `apply_splice`
- how parser support, prompt teaching, and host escalation should be separated

## Background Problem

The original attraction of context-matching patch authoring is that it reduces
the need for explicit line coordinates.

However, owner-observed host behavior has introduced an important pressure:

- ChatGPT/Codex-class agents often re-read files even after performing writes
- agents often search and inspect again after creation or modification
- therefore the expected saving from avoiding explicit line coordinates may be
  smaller in practice than the original theoretical story suggests

This does not make ordinary `apply_patch` obsolete.
It does weaken the assumption that coordinate-free authoring is always the most
economical locking contract.

## Product Direction

### Direction D1: extend `apply_patch` rather than fork its public identity

The current direction is no longer to introduce a second public patch tool.

The current direction is:

- keep one public tool identity: `apply_patch`
- extend its `Update File` chunk grammar with optional stronger locking forms
- evaluate different authoring strategies inside that one tool surface

This means the current design task is not "define a new interface".
It is "define stronger locking forms inside the existing interface without
damaging the current compatible path".

### Direction D2: the extension remains patch-shaped

The extension is not intended to become a verb-heavy transfer language.

It should preserve the high-value properties already associated with patch-shaped
editing:

- multi-file edit programs
- multi-chunk edit programs
- single network call for many coordinated edits
- file-group execution and rollback semantics

### Direction D3: the extension changes the locking contract, not the object
identity

The existing `apply_patch` object remains text difference expressed against a
target file.

The extension is currently best understood as:

- text difference with stronger old-side locking

This is different from `apply_splice`, whose object is transfer of already-
existing spans.

## Relationship to `apply_splice`

The line-locked `apply_patch` extension and `apply_splice` should be viewed as
sharing one important family resemblance:

- both favor explicit locking over open-ended context guessing

However, they do not collapse into one tool identity.

The distinction remains:

- `apply_splice`: transfer or delete existing spans without authored replacement
- line-locked `apply_patch`: authored replacement, insertion, or deletion under stronger
  old-side lock

Therefore the correct relationship is:

- shared locking philosophy where helpful
- distinct object boundary and public identity

## Three-Layer Contract Separation

One major lesson from the recent discussion is that three different layers must
be kept separate.

### Layer 1. Parser support set

This is the full syntax surface the tool can truthfully parse and execute.

### Layer 2. Prompt-preferred subset

This is the authoring form the injected tool docs should teach first because it
offers the best balance of:

- ergonomics
- reliability
- token economy
- teachability

### Layer 3. Host escalation guidance

This is the orchestration rule for when the host or model should abandon the
concise form and move to a stronger evidence pattern.

The current discussion explicitly rejects treating one of these layers as if it
automatically determined the others.

## Current Preferred Direction

### Preferred form: concise single-anchor evidence pattern

The strongest current direction is that the preferred authoring form should be a
single numbered anchor followed by ordinary diff body lines.

Illustrative shape:

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

The value of this form is that it keeps:

- patch muscle memory
- ordinary diff body semantics
- one strong old-side lock line

without forcing every old-side line to carry coordinates in the common case.

### Stronger form: denser numbered old-side evidence pattern

The current direction also preserves a stronger evidence pattern for long-tail or
ambiguity-prone situations.

This stronger pattern is not a competing worldview.
It is the higher-evidence member of the same family.

Its purpose is to handle cases such as:

- file-start insertion awkwardness
- repeated patterns after the anchor
- stronger diagnostic needs
- high-risk long chunks

## Evidence Escalation as the Main Attention-Reinforcement Strategy

One important correction from recent discussion is that the design should not
introduce a separate `CTX` mechanism as if it were a new first-class object.

The underlying need is not "a CTX header".
The underlying need is:

- additional mid-program attention reinforcement
- reduced ambiguity in long repetitive spans
- stronger tolerance under long-range autoregressive reasoning

The current design direction is to satisfy that need through:

1. stronger numbered evidence where needed
2. natural chunk decomposition where the edit already splits cleanly
3. host escalation guidance

not through a separate named mechanism detached from the existing chunk model.

More concretely:

- if a continuous local fragment still contains too much repetition after a
  concise anchor, the primary escalation path is to add stronger numbered
  old-side evidence for that chunk
- multiple chunks remain useful when the edit already decomposes into naturally
  separate local transformations
- the design should not introduce new complexity merely to avoid writing a few
  additional numbered old-side lines

## Design Principles

### Principle P1: preserve patch muscle memory where possible

The new interface should feel close enough to ordinary `apply_patch` that the
change is read as a locking upgrade, not as an unrelated language.

### Principle P2: stronger lock is more important than decorative syntax

The design should optimize for locking clarity, not for introducing visually
novel syntax markers.

### Principle P3: stronger evidence patterns exist for boundary cases, not as proof of purity

The stronger pattern exists because long-tail edge cases are real.
It should not be treated as evidence that the concise pattern failed
conceptually.

### Principle P4: newline and EOF preservation belong to the design from day one

The line-locked extension should not inherit the current formatting-takeover
defect of the existing update pipeline.

### Principle P5: same-file multi-chunk meaning must remain stable

The current direction favors:

- pre-mutation snapshot interpretation
- overlap rejection by default
- explicit diagnostics when concise locking is insufficient

## Current Open Design Set

The following points remain open and must not be written elsewhere as if they
were already closed:

1. final tool name
2. exact grammar of the stronger numbered-evidence pattern
3. exact ambiguity error contract after anchor success but body ambiguity
4. exact level of tolerated whitespace lenience in the new interface
5. whether the standalone historical `apply_patch` binary remains a product-
   level sibling or becomes clearly secondary

## What Has Already Been Rejected or Deprioritized

The following directions are currently disfavored and should not silently return
as defaults in later drafts:

1. stacked multiple `@@` hierarchy as a major investment path
2. a separate `CTX` object treated as an end in itself
3. a grammar that becomes so broad that prompt teaching and parser support drift
    apart again

## Immediate Implication for Future Spec Writing

Any future formal grammar draft for the line-locked extension must preserve the
following structure:

1. define parser support set
2. define prompt-preferred subset
3. define host escalation guidance
4. mark stronger evidence patterns explicitly

This prevents future drafts from reintroducing the same category mistake that
already hurt the current `@@` contract: treating what is parsable, what is
recommended, and what is strategically best as if they were one and the same.
