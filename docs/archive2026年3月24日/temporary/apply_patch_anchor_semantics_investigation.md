# `apply_patch` Anchor Semantics Investigation

## Status

- Recorded on 2026-03-23.
- Scope: semantic investigation of the current `@@` anchor mechanism.
- Authority level: temporary analysis record, not a stable public specification.

## Exclusions

This document does not:

- serve as the confirmed defect register
- define the final grammar of the future line-locked `apply_patch` extension
- function as an implementation schedule

Those responsibilities belong to separate artifacts.

## Purpose

This document isolates one specific design question:

> What is the current `@@` mechanism in `apply_patch`, what value does it still
> provide, and what level of further investment is justified?

The need for a dedicated investigation arises from three converging facts:

1. The active tool docs currently teach `@@` in a way that exceeds what the
   parser actually supports.
2. The current mechanism has been discussed as if it were a hierarchy or scope
   system, but the code path suggests a much narrower role.
3. The team is now evaluating stronger line-locked authoring forms inside
   `apply_patch`, which may absorb or replace some of the responsibilities
   previously projected onto `@@`.

## Evidence Base

This investigation draws on four evidence classes:

1. local parser and runtime code in `codex-apply-patch`
2. local injected tool docs in `docutouch-server/tool_docs`
3. upstream public `openai/codex` materials inspected on 2026-03-23
4. owner-observed field behavior from cross-model black-box use

The fourth class is treated as field observation rather than as parser-truth.
It is still important because the mechanism is consumed by LLMs, not by humans
reading a language manual.

## Current Parser Semantics

### One optional explicit header per chunk

In the current local parser, `parse_update_file_chunk()` accepts exactly one
optional explicit `@@` header line at the beginning of the chunk.

The relevant parser behavior is:

- `@@` means an empty explicit anchor header
- `@@ <text>` means an explicit anchor header with one line of text
- after this first optional header is consumed, the remaining chunk body must use
  ordinary diff-line prefixes

The accepted body-line prefixes remain:

- leading space for unchanged context lines
- `-` for removed lines
- `+` for added lines
- `*** End of File` where relevant

### No stacked header support

The parser does not currently support multiple consecutive `@@ ...` header lines
inside one chunk.

Operationally, a second consecutive `@@ ...` line is not interpreted as a second
anchor header. It is treated as an invalid body line.

This means the current parser contract is:

- optional single explicit anchor header
- not a chain of hierarchical headers
- not a nested scope path

## Current Runtime Semantics

### `@@` is a coarse pre-anchor, not a scope system

At runtime, `change_context` is used as a coarse search anchor before the chunk's
`old_lines` are matched.

Its effect is narrow and concrete:

1. the runtime searches the target file for the explicit anchor line
2. if found, the search cursor for later old-line matching is advanced to the
   line after that anchor
3. if not found, the chunk fails before later replacement matching proceeds

This makes the mechanism a **coarse pre-anchor**, not a semantic scope system.

It does not:

- interpret nested scopes
- understand language-level block structure
- model class/function nesting
- create a multi-level selection path

### The anchor is one line only

The current implementation treats the explicit anchor as a one-line sequence. It
does not treat `@@` as a multi-line scope declaration.

That matters because much of the historical prompt framing around `@@` suggests a
 stronger structural role than the runtime really supports.

### Whitespace lenience is matcher behavior, not `@@` magic

The runtime's sequence matcher attempts several increasingly permissive matching
passes, including whitespace-trimmed comparison. This means an anchor line may
still match when leading or trailing indentation differs.

However, this is not a special power of `@@` itself.

It is a property of the line matcher.

Therefore the following statement is false or at least misleading:

- "`@@` solves indentation as a special semantic mechanism"

The more accurate statement is:

- "the current matcher is lenient on surrounding whitespace, so a single-line
  anchor may still match under indentation differences"

## Current User-Facing Contract Drift

### Local active drift

The active injected tool doc still teaches stacked multiple `@@` header authoring
and even includes an indented example form.

This is not historical dead text. It is part of the current prompt-facing tool
surface consumed by hosts.

Therefore the current local problem is not merely that the parser is narrower
than some archived note. The current live teaching surface itself exceeds the
real parser contract.

### Upstream public drift

The same basic mismatch currently appears in upstream public materials:

- upstream tool instructions still teach multiple `@@` statements
- upstream public parser code still appears to consume only one explicit header

This upstream fact is relevant for diagnosis, but it does not justify preserving
an unhealthy local contract.

## Value Analysis

### What value the current mechanism still provides

The current single explicit anchor still has real, though bounded, value.

It provides:

1. a one-line coarse landmark before local diff matching begins
2. a way to narrow the search start without duplicating the entire landmark line
   inside the body
3. one possible target-side corroborating anchor for diagnostics

These are real values. They explain why the mechanism still feels useful in some
ordinary patches.

### What value it does not provide

The current mechanism does not justify the following stronger interpretations:

1. hierarchical scope navigation
2. class/method/inner-function path addressing
3. guaranteed ambiguity elimination by itself
4. a distinct language-theoretic substitute for ordinary context lines

Those stronger interpretations are not supported by the current code path.

## Cross-Model Consumption Risk

Field observation supplied by the project owner indicates that non-ChatGPT models
may treat `@@ ...` text more like an annotation or comment than a rigid grammar
element.

This observation matters because the mechanism's product value depends not only
on parser support, but also on whether model families reliably interpret the form
as intended.

If a mechanism:

- requires special prompt teaching
- is weakly supported by parser/runtime reality
- and is not strongly intuitive across models

then the threshold for further grammar investment should be high.

## Why Multiple Stacked `@@` Is Not the Current Priority

The current evidence does not support treating stacked multiple `@@` headers as a
high-priority improvement target.

The reasons are cumulative:

1. the parser does not support them now
2. the runtime model does not naturally point toward hierarchy semantics
3. their cross-model intuitiveness is questionable
4. ordinary context lines already cover much of the practical job
5. the team is now exploring a line-locked `apply_patch` extension that may provide a
   cleaner locking contract altogether

In other words, stacked multiple `@@` appears to be a high-complexity,
low-confidence investment path.

## Single-Anchor Reinterpretation

A much stronger local direction has emerged during discussion:

- retain `@@` only as a single anchor entry point
- do not treat it as a hierarchy mechanism
- let it carry one strong anchor role instead of many weak implied roles

This reinterpretation has three benefits:

1. it gives `@@` one clear semantic job
2. it removes the need to defend stacked headers as a product priority
3. it aligns more naturally with the future concise anchor form being considered
   for the line-locked `apply_patch` extension

## Consequences for Local Documentation

The current local prompt surface should stop teaching stacked multiple `@@`
header authoring as if it were a normal path.

Until or unless parser support actually changes, the healthy local contract is:

- one explicit anchor at most
- ordinary context lines remain the main local disambiguation mechanism

This does not require banning every currently parseable variation.
It does require stopping the active teaching of forms the runtime rejects.

## Relationship to the Future Line-Locked `apply_patch` Extension

This investigation does not claim that `@@` should disappear from all future
designs.

What it does show is:

- the current `@@` mechanism should not be treated as a foundation for a rich
  hierarchy language
- if `@@` survives into the future line-locked extension, its most credible role
  is a single numbered anchor entry point, not a stacked header chain

This is a design implication, not yet a final grammar decision.

## Conclusion

The current `@@` mechanism is best understood as:

- a single optional coarse anchor line
- used to advance the later search window
- helpful in bounded cases
- not a hierarchy system
- not a parser-backed multi-header path language

The most important local action implied by this investigation is not to expand
stacked multiple `@@` support immediately.

It is to:

1. document the current mechanism truthfully
2. remove active prompt/runtime drift
3. carry forward only the part of `@@` that still has a clean role

That clean role is single-anchor entry, not stacked hierarchy.
