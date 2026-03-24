# CLI Adapter Implementation Plan

## Status

- Scope: detailed implementation plan for the Rust CLI adapter
- Complements `docs/cli_adapter_spec.md`

## Phase 0. Design Lock

Status: completed

Tasks:

- [x] Confirm that the CLI should be an adapter, not a second semantics layer
- [x] Confirm that CWD replaces explicit workspace negotiation in the CLI flow
- [x] Separate design specification from execution planning

## Phase 1. Shared-Layer Audit

Status: completed

Tasks:

- [x] Identify which MCP behaviors already live below the server layer
- [x] Identify which `search_text` responsibilities still live in `docutouch-server`
- [x] Identify which `apply_patch` presentation responsibilities still live in `docutouch-server`
- [x] Define extraction targets before any large CLI surface is added

Deliverable:

- implementation notes for transport-agnostic semantic extraction

## Phase 2. Shared Semantic Extraction

Status: completed

Tasks:

- [x] Extract `search_text` semantics/rendering into a shared layer
- [x] Extract `apply_patch` success/failure presentation into a shared layer where appropriate
- [x] Centralize path display helpers that both MCP and CLI should share
- [x] Keep behavior unchanged while moving ownership downward

Deliverable:

- thinner MCP server layer with shared semantic utilities available for CLI use

## Phase 3. CLI Surface Implementation

Status: completed

Tasks:

- [x] Add a `docutouch` CLI binary
- [x] Implement `list`, `read`, `search`, and `patch` subcommands
- [x] Use CWD as the implicit relative-path anchor
- [x] Support stdin patch ingestion for `patch`
- [x] Support one-or-more path arguments for `search`

Deliverable:

- initial parity-oriented CLI surface

## Phase 4. Parity and Regression Testing

Status: completed for the current planned parity scope

Tasks:

- [x] Add parity tests for `search` preview/full behavior
- [x] Add parity tests for `patch` success and partial failure output
- [x] Add transport-specific tests for CWD semantics and stdin patch handling
- [x] Verify diagnostics and warning parity where expected

Deliverable:

- automated confidence that CLI and MCP have not drifted

## Phase 5. Documentation and UX Polish

Status: completed for README / roadmap level adapter documentation

Tasks:

- [x] Document CLI invocation without inventing a second product vocabulary
- [x] Document CWD anchoring rules clearly
- [x] Clarify the relationship between `docutouch patch` and the standalone `apply_patch` binary
- [x] Check whether tool docs and README need parity notes

Deliverable:

- user-facing CLI documentation aligned with the adapter model

## Phase 6. Validation and Acceptance Review

Status: completed

Tasks:

- [x] Run targeted CLI tests
- [x] Re-run relevant MCP tests to confirm no regressions from extraction
- [x] Review whether any duplicated semantics remain
- [x] Prepare ship-readiness summary

Deliverable:

- implementation review and acceptance package

## Review Standard

- adapter over duplication
- semantic parity first
- CWD as implicit workspace anchor

## Progress Log

- 2026-03-20: CLI adapter design specification created
- 2026-03-20: execution plan separated from the design specification
- 2026-03-20: shared `search_text` / patch presentation logic extracted into `docutouch-core`
- 2026-03-20: `docutouch` binary now supports `list/read/search/patch` while keeping no-arg stdio server startup
- 2026-03-20: parity-oriented CLI tests added for search preview, patch success, patch partial failure, and CWD-based read behavior
- 2026-03-20: sampled `read` support integrated across CLI and MCP after the shared adapter work landed
- 2026-03-20: main-agent integration review completed; CLI adapter accepted for the current documented parity scope
