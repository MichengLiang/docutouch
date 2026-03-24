# `search_text` Implementation Plan

## Status

- Owner: main agent
- Implementation worker: `gpt-5.4` / high
- Main-agent role: product decisions, QA, review, integration, doc promotion
- Worker role: code changes and tests in server implementation

## Scope Summary

This plan tracks the implementation of the accepted `search_text` UX contract.

Target outcomes:

- add `view = preview | full`
- support `path: string | string[]`
- adopt `render-shaping flags` taxonomy
- reject context flags until they are explicitly modeled
- teach the contract in prompt-facing tool descriptions
- change default ranking to relevance-first deterministic ordering
- make `preview` an explicit overview representation

## Phase 0. Contract Lock

Status: completed

Tasks:

- [x] Promote RFC draft into formal docs as `docs/search_text_ux_contract.md`
- [x] Record two-view model: `preview` and `full`
- [x] Record `render-shaping flags` terminology
- [x] Record multi-path union-scope decision under the existing `path` field

Notes:

- Product decision: keep the public field name `path` for backward compatibility.
- Product decision: allow `path` to accept a single string or an array of strings.

## Phase 1. Input Contract and Parsing

Status: completed

Tasks:

- [x] Extend `SearchTextArgs` to support `path: string | string[]`
- [x] Add `view?: preview | full` with `preview` as the default
- [x] Normalize union-scope resolution across files and directories
- [x] Preserve existing single-path compatibility
- [x] Update tests for valid and invalid argument shapes

Review focus:

- no ambiguity between single-path and multi-path behavior
- error messages remain teachable

## Phase 2. Flag Taxonomy and Prompt-Facing Guidance

Status: completed

Tasks:

- [x] Separate `search-behavior flags` from `render-shaping flags`
- [x] Extend rejection set to include context flags
- [x] Update tool description text in server registration
- [x] Update parameter description text for prompt injection quality
- [x] Ensure rejection errors explain the contract, not just the failure

Review focus:

- tool docs should help the model avoid errors before first use
- terminology must stay consistent across code and docs

## Phase 3. Rendering Modes

Status: completed

Tasks:

- [x] Implement `preview` grouped overview rendering
- [x] Implement `full` grouped exhaustive rendering
- [x] Add omission accounting in `preview`
- [x] Preserve grouped-by-file output in both modes
- [x] Keep `full` non-raw and non-CLI-mirror in shape

Review focus:

- `preview` must be explicit overview, not accidental truncation
- `full` must remain stable and grouped

## Phase 4. Ranking and Scope Presentation

Status: completed

Tasks:

- [x] Replace path-first ordering with relevance-first deterministic ordering
- [x] Render `scope` coherently for single-path and multi-path input
- [x] Verify ordering is stable under ties
- [x] Confirm grouped counts still match the raw result set

Review focus:

- ranking should improve first-pass utility without introducing unstable magic

## Phase 5. Test and QA Pass

Status: completed

Tasks:

- [x] Add tests for `preview` and `full`
- [x] Add tests for path-array union scope
- [x] Add tests for `-n` rejection
- [x] Add tests for `-C` rejection
- [x] Add tests for ranking behavior
- [x] Run targeted Rust tests

Review focus:

- verify contract behavior, not just implementation internals

## Phase 6. Integration Review

Status: completed

Tasks:

- [x] Review worker diff against the UX contract
- [x] Read changed tests and confirm they reflect the approved model
- [x] Update this plan with completion notes
- [x] Prepare ship/readiness summary for user handoff

## Progress Log

- 2026-03-20: RFC draft accepted and promoted to official doc.
- 2026-03-20: Multi-path union-scope decision accepted under existing `path` field.
- 2026-03-20: Worker task delegated for implementation and test updates.
- 2026-03-20: `search_text` implementation completed in server code and tests.
- 2026-03-20: Main-agent QA review completed; no blocking findings discovered.
- 2026-03-20: `cargo test -p docutouch-server` passed after the contract update.

## Residual Notes

- `preview` uses internal rendering limits to form the explicit overview view.
- Long-line snippet windowing is still a follow-up enhancement, not part of this pass.
- Overlapping multi-root scopes may still duplicate underlying file hits if ripgrep emits the same file through different roots; this was left as a conscious non-goal for the current phase.
