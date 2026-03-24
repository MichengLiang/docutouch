# `read_file` Sampled View Implementation Plan

## Status

- Scope: detailed implementation plan for sampled inspection mode in `read_file`
- Complements `docs/read_file_sampled_view_spec.md`

## Phase 0. Design Lock

Status: completed

Tasks:

- [x] Confirm that sampled view is a confidence-oriented inspection mode
- [x] Reject DSL-based control in v1
- [x] Confirm the explicit parameter model: `sample_step`, `sample_lines`, `max_chars`
- [x] Confirm vertical `...` and inline `...[N chars omitted]` as the two omission forms
- [x] Confirm that sampled mode does not prepend an out-of-band metadata header

## Phase 1. Argument Contract Design

Status: completed

Tasks:

- [x] Decide final argument names and schema placement
- [x] Define the exact activation rule for sampled mode
- [x] Define validation behavior for invalid parameter combinations
- [x] Define line-number guidance without forcing or silently overriding the flag

Deliverable:

- final API contract for sampled mode

## Phase 2. Core Rendering Logic

Status: completed

Tasks:

- [x] Implement sampled block selection over a bounded contiguous range
- [x] Render standalone vertical omission markers
- [x] Render inline horizontal truncation markers with omitted char counts
- [x] Ensure sampled blocks remain ordered and non-overlapping

Deliverable:

- a deterministic sampled-view renderer

## Phase 3. Prompt-Facing Guidance

Status: completed

Tasks:

- [x] Document sampled mode as a dense local confidence-check tool
- [x] Add recommended parameter combinations to the tool description
- [x] Keep the prompt guidance focused on cognitive intent rather than free-form tuning

Deliverable:

- prompt-visible guidance for practical use

## Phase 4. Validation and Edge Cases

Status: completed

Tasks:

- [x] Test very short ranges
- [x] Test ranges that are not divisible by `sample_step`
- [x] Test long lines with `max_chars`
- [x] Test interactions between sampled mode and ordinary exact range behavior

Deliverable:

- confidence that the sampled view remains non-ambiguous across edge cases

## Phase 5. Review and Acceptance

Status: completed

Tasks:

- [x] Review whether omission semantics remain obvious at a glance
- [x] Review whether sampled mode is still clearly distinct from exact reading
- [x] Prepare ship-readiness summary

Deliverable:

- implementation review and acceptance package

## Review Standard

- confidence-oriented, not compression-oriented
- vertical and horizontal omission remain type-distinct
- sampled mode stays low-surprise

## Progress Log

- 2026-03-20: sampled-view design specification created
- 2026-03-20: execution plan separated from the design specification
- 2026-03-20: lower-layer sampled rendering landed in `docutouch-core`
- 2026-03-20: MCP `read_file` and CLI `docutouch read` now expose sampled-view parameters
- 2026-03-20: parity and smoke tests passed for sampled view across core, server, and CLI
