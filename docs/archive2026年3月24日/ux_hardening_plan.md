# DocuTouch UX Hardening Plan

## Purpose

This document turns the current UX discussion into an implementation program.
It is not limited to one isolated patch fix. It lays out the full DocuTouch UX
surface that should be tightened, in dependency order, until the toolset feels
coherent instead of half-explicit and half-ambient.

## Scope

The scope of this plan covers:

- `apply_patch` diagnostics and repair UX
- workspace / cwd behavior across tools
- output consistency between tools and layers
- host-audit boundary and failed-patch-source repair UX
- low-noise success and no-op behavior
- documentation synchronization for the real runtime contract

It does not attempt to redesign the entire product or add broad new tool
categories.

## Current Status Snapshot

### Already implemented

- `apply_patch` success and partial-failure flows now preserve committed
  `A/M/D` changes.
- failed file groups now surface structured diagnostics:
  - stable failure code
  - target path
  - action index
  - hunk index when available
  - attempted `A/M/D`
  - help text
- partial-success summaries now count committed file groups instead of file
  paths.
- malformed `Add File` content lines now report a more precise outer-format
  error.
- durable audit and traceability are now explicitly host-owned rather than
  duplicated inside DocuTouch.
- net-zero `Update File` patches are confirmed to elide writes and preserve file
  timestamps.
- the following docs have been synchronized to the current state:
  - `README.md`
  - `docs/product_positioning.md`
  - `docs/apply_patch_semantics_plan.md`
  - `docutouch-server/tool_docs/apply_patch.md`
  - `codex-apply-patch/DOCUTOUCH_ENHANCEMENTS.md`
- Wave 0.5 is now implemented:
  - `read_files` is fully decommissioned as a callable tool
  - server dispatch / parsing / tests for `read_files` are removed
  - the unused `docutouch-core` batch-read implementation is removed
  - `docutouch-server/tool_docs/read_files.md` now serves as a deprecation record
- Wave 1 is now implemented:
  - `set_workspace` remains the first-class runtime override
  - startup now honors a valid `DOCUTOUCH_DEFAULT_WORKSPACE`
  - relative-path precedence is now explicit `set_workspace` -> env default -> error
  - `apply_patch` no longer uses process cwd as the semantic default workspace
- Wave 2 is now implemented:
  - execution-time failures now carry one primary patch-source location when available robustly
  - context-match failures point to the relevant update chunk start
  - action-level failures such as missing update targets point to the relevant patch action line
- Wave 3 as originally written landed historically, but that direction is now superseded in part:
  - audit and traceability still belong to host-level tool-call receipts rather than a duplicated tool-layer cache
  - audit-shaped failure artifacts, patch-run caches, and secondary JSON reports remain outside the desired product contract
  - failure-time patch source persistence is now treated as a legitimate repair object when the patch did not already come from a stable file path
- Wave 4 is implemented for the currently planned parity scope:
  - standalone CLI now prints `(no file changes)` for no-op success just like the server
  - failure wording stays compact while remaining more consistent about source locations and the repair contract

### Still unresolved

- execution-time diagnostics still stop at a single primary location; they do not
  attempt richer multi-span rendering
- partial-failure presentation still compresses some committed lists and still mixes
  transaction-level accounting with first-failure-local fields
- failed patch source persistence, per-group patch pointers, and multiline cause
  indentation still need to be synchronized into runtime behavior and tests
- some historical docs still quote the retired artifact layer and should stay
  clearly marked as superseded history rather than active direction
- search is still terminal-first rather than tool-first; there is no dedicated
  grouped search surface yet for high-frequency text lookup

## UX Problem Inventory

### U1. Workspace model inconsistency

Symptoms:

- users must remember which tools honor ambient cwd and which require explicit
  workspace state
- the current model encourages accidental cargo-culted `set_workspace` calls
- the contract is harder to teach and harder for agents to internalize

Why this matters:

- this is now the largest remaining UX inconsistency in the toolset
- it affects every task, not just patching

### U2. Execution diagnostics are better, but not yet source-span grade

Symptoms:

- parse errors can point at patch lines directly
- execution failures can now report action / hunk, but still do not reliably map
  back to exact patch-source spans

Why this matters:

- this is the most rustc-like next step that still fits DocuTouch's low-noise
  design philosophy

### U3. Historical tool-managed recovery artifacts duplicated host audit logs

Symptoms before Wave 3 retirement:

- the tool wrote `.docutouch/patch-runs` artifacts on failure
- failure messages mentioned secondary files that neither the model nor human
  reviewers used in the current workflow
- the Codex host already retains the full tool-call receipt stream, making the
  tool-layer copy redundant

Why this matters:

- duplicated audit layers create maintenance cost without creating real repair
  value
- a second persistence layer inside the tool conflicts with the current belief
  that diagnostics must be self-contained
- host-level observability is the correct layer for durable audit trails

### U4. No-op semantics are correct, but not fully surfaced as contract

Symptoms:

- runtime behavior is good: no write, no timestamp churn
- docs and CLI/server parity are not fully finished

Why this matters:

- invisible no-op writes would be catastrophic UX
- the current safe behavior should become an explicit guarantee

### U5. Search remains more expensive than it should be

Symptoms:

- raw terminal `rg` is powerful but emits repeated path tokens and flat match
  streams
- high-hit searches still produce more token noise than is ideal for an LLM
- there is no grouped, overview-first search contract that naturally feeds
  `read_file`

Why this matters:

- search is part of the main path, not an edge workflow
- the next meaningful UX gain after the current patch/repair work is likely to
  come from a better search surface rather than more file-reading primitives

### U6. `read_files` sat in an ambiguous product state

Symptoms before Wave 0.5:

- implementation and docs remain
- registration is removed
- some surrounding docs historically drifted

Why this mattered:

- ambiguity causes wrong expectations
- the current state is worse than either full support or full removal
- product direction is to remove the tool entirely and keep only a deprecation
  record explaining why ordinary `read_file` calls replaced it

## Program of Work

### Wave 0. Contract Cleanup

Goal:

- eliminate stale or self-contradictory documentation before more UX work lands

Tasks:

- keep `docs/apply_patch_semantics_plan.md` aligned with implemented behavior
- keep `docutouch-server/tool_docs/apply_patch.md` aligned with runtime reality
- keep high-level docs explicit about the decommissioned `read_files` state
- preserve the rule that temporary testing and search stay inside the current
  project or submodule

Status:

- complete

Remaining:

- update this plan as later waves land

### Wave 0.5. `read_files` Decommission

Goal:

- eliminate the current hidden-but-still-callable `read_files` state

Tasks:

- remove the `read_files` route from the server dispatch path
- remove the server-side request parsing and implementation
- remove server tests that still exercise the hidden route
- decide whether any `docutouch-core` support should remain for internal use
- replace active tool-doc framing with a deprecation record that captures:
  - why `read_files` existed
  - why it was removed
  - why repeated ordinary `read_file` calls are preferred
  - why single-call giant payloads are a bad fit for modern hosts

Expected impact:

- smaller tool surface
- less hidden behavior
- lower maintenance burden

Risk:

- low
- the main requirement is to preserve a clear historical rationale in docs

Status:

- implemented

### Wave 1. Workspace Semantics Unification

Goal:

- remove the cross-tool split between explicit workspace and implicit ambient
  behavior

Recommended product rule:

- support a startup environment variable such as
  `DOCUTOUCH_DEFAULT_WORKSPACE`
- keep `set_workspace` as a first-class tool:
  - it remains the explicit runtime workspace switch
  - it also keeps the path model visible to the LLM and user
- if explicit `set_workspace` has been called, use that
- else if `DOCUTOUCH_DEFAULT_WORKSPACE` is present and valid, use it as the
  initial workspace
- else require explicit workspace for relative paths
- absolute paths always bypass workspace

Implementation tasks:

- remove `apply_patch`'s direct fallback to `std::env::current_dir()`
- teach the server to load and validate a startup default workspace from env
- audit all relative-path tools for the same precedence rule
- make the behavior consistent
- update tests for unset-workspace flows
- add tests for env-provided default workspace flows
- if the env variable is present but invalid:
  - keep the server startable
  - treat it as “no default workspace”
  - emit a low-noise warning / log
  - keep relative-path failures precise and action-guiding
- document the final rule in tool docs and README-level docs

Expected impact:

- lower friction on the main path
- simpler mental model for both humans and agents
- no dependence on undocumented host cwd propagation behavior

Risk:

- medium product risk, low implementation risk
- this changes a core contract and should be communicated clearly
- invalid env configuration must be visible, but should not block server startup

Status:

- implemented

### Wave 2. Source-Span Grade Execution Diagnostics

Goal:

- make execution failures feel one level closer to rustc without becoming noisy

Tasks:

- carry exact patch-source line information through execution-time failures when
  possible
- distinguish action-level and chunk-level failure loci more precisely
- render a single primary span, not a wall of duplicated context
- keep the message shape stable:
  - headline
  - location
  - concise cause
  - repair-oriented help

Expected impact:

- faster self-repair loops
- less wasted re-read / regenerate churn

Risk:

- medium implementation complexity
- must avoid turning diagnostics into a verbose terminal art project

Status:

- implemented

### Wave 3. Recovery Artifact Retirement

Goal:

- retire tool-managed failure artifacts so the inline failure surface fully owns
  ordinary repair loops

Tasks:

- remove `.docutouch/patch-runs` writing from ordinary failure paths
- remove artifact notes from inline diagnostics
- move the product contract to: inline failure is sufficient; host logs handle
  audit / replay history outside the tool

Expected impact:

- clearer tool responsibility boundaries
- lower maintenance burden and less duplicated audit machinery
- stronger pressure to keep the inline failure surface complete

Risk:

- medium
- docs, tests, and implementation must all be synchronized or the repository will
  drift into contradiction

Status:

- implemented

### Wave 4. Success / No-op / CLI Parity Polish

Goal:

- close the remaining polish gaps after correctness and contract work

Tasks:

- consider aligning standalone CLI no-op success messaging with server behavior
- review whether success warnings and failure diagnostics share enough visual
  consistency
- verify failed-patch-source wording and host-audit boundary remain transport-
  consistent after the 2026-03-23 contract change

Expected impact:

- modest but worthwhile finish quality

Risk:

- low
- should not preempt higher-value core consistency work

Status:

- implemented for the currently planned scope

### Wave 5. `search_text` Design and Implementation

Goal:

- add a grouped ripgrep-backed search surface with lower token noise than raw
  terminal output

Tasks:

- keep raw `rg` available in the terminal as the unrestricted escape hatch
- design `search_text` around grouped-by-file output rather than a flat match
  stream
- keep the public contract minimal and high-ROI
- allow raw ripgrep passthrough only as an escape hatch, not as the main
  teaching path
- ensure search results naturally point toward `read_file` follow-up calls

Expected impact:

- cheaper high-frequency search flows
- lower repeated-path noise
- better “overview first, expand second” agent workflows

Risk:

- medium
- it is easy to accidentally build either a disguised CLI mirror or a hidden
  truncation machine

Current design note:

- see `docs/search_text_design.md`

## Priority Order

Recommended order:

1. Wave 0.5: `read_files` decommission
2. Wave 1: workspace semantics unification
3. Wave 2: source-span grade execution diagnostics
4. Wave 3: recovery artifact retirement
5. Wave 4: parity polish
6. Wave 5: `search_text`

Reasoning:

- Wave 0.5 removes a hidden and misleading tool surface with little downside
- Wave 1 removes the biggest remaining cross-tool UX contradiction
- Wave 2 deepens the highest-value diagnostics improvement
- Wave 3 removes a redundant tool-layer audit cache once the visible contract is
  stable
- Wave 4 is polish and should not block structural gains
- Wave 5 is the next likely high-ROI main-path enhancement after patch/workspace
  stabilization

## What “Done” Should Mean

This plan is only complete when:

- all high-frequency tools share one coherent workspace model
- `apply_patch` execution failures are precise enough that the next repair step
  is obvious
- partial-failure reporting is stable, structured, and low-noise
- no-op patch behavior is contractually documented and regression-tested
- docs describe the real runtime, not an aspirational or outdated one

## Notes

- The goal is not to mimic rustc cosmetically. The goal is to reach rustc-like
  clarity under an agent-native, token-sensitive interface.
- The goal is also not to choose one UX improvement and discard the rest.
  The whole program should ship; this document only fixes the order of attack.
