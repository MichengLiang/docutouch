# CLI Adapter Detailed Design

## Status

- Detailed design specification
- Intended to guide the CLI adapter implementation for the Rust DocuTouch stack

## 1. Problem statement

DocuTouch currently exposes its primary product surface through MCP tools.

That surface is now increasingly mature:

- `list_directory`
- `read_file`
- `search_text`
- `apply_patch`

However, the project does not yet expose the same product surface as a native
CLI with parity semantics.

The need is not to invent a second product.
The need is to expose the same core capabilities through a second transport and
invocation surface.

## 2. Background

The current architecture already contains reusable lower layers:

- `codex-apply-patch`
  authoritative patch grammar, patch execution, diagnostics, summary generation
- `docutouch-core`
  reusable filesystem primitives and patch-runtime shaping
- `docutouch-server`
  MCP registration, schema definitions, workspace negotiation, transport-specific
  rendering glue

The important observation is that the project is not starting from zero.
Several semantics are already implemented below the MCP transport boundary.

At the same time, not all product behavior is fully transport-agnostic yet.
Notably, some search and presentation logic still lives in the server layer.

Therefore the central design challenge is not "how to parse CLI args".
The central design challenge is:

- how to expose CLI parity without duplicating product semantics

## 3. Primary design principle

The CLI must be an adapter, not a second product semantics layer.

This means:

- the CLI should reuse the same semantic core as MCP whenever possible
- the CLI should not reimplement search or diagnostics behavior in parallel
- transport-specific differences should stay limited to invocation shape and path
  anchoring rules

The goal is semantic parity, not superficial feature resemblance.

## 4. Product goals

The CLI adapter should:

- expose the MCP tool surface through native command-line subcommands
- preserve the same result semantics and output grammar wherever possible
- remove explicit workspace negotiation from the user-visible CLI flow
- keep relative-path behavior predictable via process CWD
- keep the MCP server and CLI from drifting into separate product dialects

## 5. Non-goals

The CLI adapter should not:

- become a raw shell toolbox that bypasses DocuTouch semantics
- mirror every low-level program flag as a first-class CLI flag
- create a second independent implementation of `search_text` or `apply_patch`
- replace MCP as the primary integration surface for LLM hosts
- turn CWD semantics into an excuse to weaken path clarity

## 6. Core product decisions

### Decision D1: the CLI is a transport adapter, not a new semantic layer

Reasoning:

- duplicated semantics are expensive to maintain
- any product rule duplicated across server and CLI will drift over time
- parity is more valuable than transport-specific flourish in v1

### Decision D2: CLI removes explicit workspace negotiation, but not path anchoring semantics

The CLI does not need a `set_workspace` command.
That does not mean the concept of path anchoring disappears.

Instead:

- process CWD becomes the implicit workspace anchor for relative paths
- absolute paths remain valid
- rendered paths should still prefer compact relative display when possible

This should be understood as:

- no explicit workspace protocol step
- one implicit workspace anchor: CWD

### Decision D3: extract shared semantics before building a large CLI surface

The first implementation wave should prioritize moving transport-agnostic logic
out of `docutouch-server` where necessary.

This especially matters for:

- `search_text` behavior and rendering
- `apply_patch` success/failure presentation

Without this extraction, the project will accumulate two copies of the same
product behavior.

### Decision D4: preserve a one-to-one conceptual mapping with the MCP toolset

The CLI should present subcommands that directly correspond to the MCP tools.

Recommended conceptual mapping:

- `docutouch list`
- `docutouch read`
- `docutouch search`
- `docutouch patch`

This does not require the positional CLI syntax to look like JSON.
It does require the mental model and result surface to remain aligned.

### Decision D5: parity tests are mandatory, not optional polish

The adapter should not be considered complete without explicit parity testing
between MCP and CLI outputs for representative scenarios.

## 7. Semantic parity contract

The CLI should preserve the same semantics as MCP for the following dimensions:

- path resolution behavior, adjusted only by replacing explicit workspace with CWD
- output grammar
- error classification
- warning behavior
- search ranking and omission logic
- patch diagnostics and partial-failure repair accounting

Expected tolerated differences:

- invocation shape
- absence of `set_workspace`
- path display rooted in current CWD rather than negotiated workspace

## 8. Command surface design

### 8.1 `docutouch list`

Maps to `list_directory`.

Expected responsibilities:

- accept an optional path argument, defaulting to `.`
- preserve depth / hidden / gitignored / timestamp options as appropriate
- keep ASCII tree output and metadata layout aligned with MCP output

### 8.2 `docutouch read`

Maps to `read_file`.

Expected responsibilities:

- accept a file path
- optionally expose line range
- optionally expose line numbers
- keep result text aligned with MCP behavior

### 8.3 `docutouch search`

Maps to `search_text`.

Expected responsibilities:

- accept a query
- accept one or more scope paths
- preserve `preview` / `full` view behavior
- preserve `rg_args` taxonomy
- preserve ranking and omission behavior

Important CLI-specific note:

The MCP contract currently models scope through `path: string | string[]`.
The CLI should naturally express the same concept via one or more path
arguments.

This is a transport convenience, not a semantic divergence.

### 8.4 `docutouch patch`

Maps to `apply_patch`.

Expected responsibilities:

- accept patch text through stdin and optionally through a file argument
- preserve the current success summary shape
- preserve warning and failure diagnostics
- preserve partial-failure repair accounting through the inline message alone

Important note:

The existing standalone `apply_patch` binary may remain useful.
`docutouch patch` should be understood as the unified CLI surface for the full
DocuTouch tool family, not as a reason to remove the existing binary.

## 9. Shared-layer extraction targets

The following logic should be considered transport-agnostic and therefore strong
candidates for extraction or centralization:

### 9.1 Search semantics and rendering

Shared responsibilities:

- scope resolution normalization
- `rg_args` validation rules
- `preview` / `full` rendering
- ranking behavior
- omission accounting

### 9.2 Patch presentation

Shared responsibilities:

- success summary formatting
- warning block formatting
- failure headline and evidence layout
- partial-failure rendering
- host-audit-neutral failure rendering with no secondary-file requirements

### 9.3 Path display logic

Shared responsibilities:

- compact display of paths relative to the current anchor
- consistent normalization across CLI and MCP

## 10. Why CWD is enough for the CLI anchor

The CLI does not need explicit workspace negotiation because the shell already
provides process CWD.

This is sufficient if the implementation preserves the following rules:

- relative paths resolve against CWD
- absolute paths bypass CWD as usual
- display paths can be compacted relative to CWD when useful

This is a product simplification, not a semantic deletion.

## 11. Testing strategy

The adapter requires more than unit tests.
It needs parity-oriented tests.

### 11.1 Parity tests

Representative scenarios should verify that MCP and CLI produce equivalent
behavior for the same underlying operation.

Examples:

- `search_text` preview/full parity
- `apply_patch` success parity
- `apply_patch` partial-failure parity
- `apply_patch` diagnostics parity
- `list_directory` and `read_file` formatting parity

### 11.2 Transport-specific tests

Additional tests should cover:

- positional argument parsing
- stdin patch ingestion
- CWD-based relative path behavior
- multiple search roots from CLI path arguments

## 12. Migration strategy

The implementation should proceed in stages.

### Stage A. Shared semantic extraction

Goal:

- reduce server-owned behavior that the CLI would otherwise duplicate

### Stage B. Thin CLI surface

Goal:

- add a binary with subcommands that mostly delegate into shared semantic code

### Stage C. Parity validation

Goal:

- confirm that CLI and MCP remain semantically aligned

### Stage D. Documentation and polish

Goal:

- document CLI invocation without rewriting the product model

## 13. Acceptance criteria

The CLI adapter should be considered successful only if:

- the command surface is conceptually aligned with MCP tools
- CWD cleanly replaces explicit workspace negotiation for CLI use
- shared semantics are not duplicated in parallel unnecessarily
- parity tests exist for the highest-value user flows
- warning / diagnostics / ranking behavior remain aligned across transports

## 14. Relationship to scheduling

This document is intentionally not the schedule.

The schedule belongs in a separate temporary execution plan so that design and
execution order do not collapse into one file.
