# `read_file` Sampled View Detailed Design

## Status

- Detailed design specification
- Intended to guide the sampled inspection extension for `read_file`

## 1. Problem statement

`read_file` currently supports exact contiguous slices through `line_range` and
optional line-number rendering.

That works well for:

- precise local inspection
- patch authoring support
- exact contextual re-reading

It is less optimized for a different, high-frequency behavior:

- low-cost confidence checks after writing or restructuring a file

In that behavior, the caller does not always want a full contiguous reread.
The caller wants a compact, semantically explicit inspection surface that is
cheaper than full rereading but still trustworthy.

## 2. Core product idea

The new mode should be understood as a dense local inspection view, not as a
general summarizer and not as a fuzzy preview.

It exists to support a specific cognitive activity:

- “I do not need to reread everything, but I want enough evidence to feel safe
  proceeding.”

This is not pure token minimization.
It is confidence-oriented inspection.

## 3. Non-goals

The sampled view should not:

- replace exact contiguous `line_range`
- become a mini programming language for arbitrary file sampling strategies
- become a whole-file summarizer
- hide omission semantics behind implicit formatting tricks
- replace deep semantic reading for debugging or patch authoring

## 4. Product decisions

### Decision D1: no DSL in v1

The sampled view should not introduce a string-embedded sampling DSL in v1.

Reasoning:

- explicit parameters are easier to understand than symbolic mini-languages
- sampled inspection is a distinct mode, not an extension of exact slicing
- the primary goal is minimal surprise, not compact syntax cleverness

### Decision D2: use explicit sampled-view parameters

The sampled view should be controlled with explicit parameters rather than a
slice-string grammar.

Recommended parameters:

- `sample_step`
- `sample_lines`

### Decision D3: the mode is local and dense, not sparse and global

The intended use is bounded local inspection after the caller has already chosen
a range worth checking.

This means:

- small steps are expected
- repeated omission markers are acceptable inside a bounded range
- the mode is not intended as a cheap proxy for whole-file reading

### Decision D4: line numbers remain a recommendation, not a forced mode switch

Sampled mode should not silently force line numbers on.

Reasoning:

- `read_file` remains a content-first surface
- line numbers are still a user-controlled reading aid
- many sampled-view use cases are low-cost relevance or structure checks rather
  than exact audit reads
- when the caller needs stronger positional certainty, it can explicitly enable
  line numbers

### Decision D5: vertical omission remains explicit

The system must render omitted whole lines with a standalone marker line.

## 5. Proposed external contract

The exact parameter names can still be refined, but the recommended external
shape is:

```text
read_file(
  relative_path: string,
  line_range?: range,
  show_line_numbers?: bool,
  sample_step?: positive integer,
  sample_lines?: positive integer
)
```

### 5.1 Parameter semantics

- `line_range`
  - continues to define the bounded region under inspection
  - remains the exact contiguous selector

- `sample_step`
  - every N lines, begin a sampled block
  - enables sampled mode when present

- `sample_lines`
  - number of consecutive lines shown in each sampled block

### 5.2 Validation rule

The implementation should treat the following as the meaningful sampled-mode
shape:

- `1 <= sample_lines < sample_step`

This rule keeps the mode genuinely sampled rather than collapsing toward
near-contiguous reading.

## 6. Output contract

### 6.1 No out-of-band metadata header

`read_file` should remain a content-first surface.

Therefore sampled mode should not prepend a separate metadata header such as:

- `sampled view`
- `range: ...`
- `sample_step: ...`
- `sample_lines: ...`

The caller already knows the requested parameters.
The returned body should consist only of the transformed text view itself.

### 6.2 Vertical omission

Vertical omission should use a standalone line:

```text
...
```

Its meaning is determined by the surrounding numbered lines.

Example:

```text
120 | def build_context(...)
...
124 | context = {
```

This is strongest when line numbers are enabled, but still acceptable without
them because sampled mode is a heuristic inspection surface, not a claim of
continuous exact reading.

## 7. Recommended reading model

The sampled view should be explained as a confidence-oriented inspection mode.

It is best suited for:

- recently written files where global context is still fresh
- repetitive or structurally regular files
- low-cost structural verification before proceeding
- quick relevance checks before deciding whether to reread a local region fully

It is not the preferred tool for:

- exact patch authoring
- deep semantic debugging
- subtle formatting or off-by-one investigation

## 8. Golden parameter recommendations

The tool description should recommend parameter combinations in terms of
cognitive intent, not only raw compression.

### Recommended set A: balanced local check

- `sample_step = 4`
- `sample_lines = 2`

This gives a dense local inspection surface with moderate vertical thinning.

### Recommended set B: cheaper local check

- `sample_step = 5`
- `sample_lines = 2`

This is the preferred recommendation for ordinary post-write confidence checks.

### Recommended set C: conservative local check

- `sample_step = 3`
- `sample_lines = 2`

This keeps more local continuity for formatting-sensitive or structure-sensitive
content.

## 10. Prompt-facing guidance

Because this mode is intended for fast inspection rather than free-form tuning,
the tool description should teach the mode through recommended patterns.

Recommended prompt-facing phrasing:

- sampled view is for dense local confidence checks, not full reading
- use small `sample_step` values such as `3-5` on a bounded range
- keep `sample_lines=2` for a good balance between continuity and compactness
- enable line numbers when the task is precision-sensitive; leave them off when
  the goal is only low-cost structural or relevance checking

## 11. Acceptance criteria

The sampled view should count as successful only if:

- the caller can distinguish vertical omission at a glance
- the mode stays obviously different from exact contiguous reading
- the recommended parameter combinations feel stable and unsurprising
- sampled mode improves confidence-check workflows without becoming a second
  miniature reading language

## 12. Relationship to future work

Potential future questions, intentionally excluded from v1:

- whether sampled mode should support whole-file structural skim presets
- whether display-width accounting should move from character count toward true
  display columns
- whether omission counts should sometimes be rendered more explicitly than a
  bare vertical `...`

These should remain follow-up decisions, not reasons to overload the first
design pass.
