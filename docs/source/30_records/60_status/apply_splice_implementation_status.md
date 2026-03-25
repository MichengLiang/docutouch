(records-status-apply-splice-implementation)=
# Apply Splice Implementation Status

## Role

本页记录 `apply_splice` 从 baseline-lock 进入真实实现后的当前执行状态。

## Current Standing

| Work Item | Current Standing | Note |
| --- | --- | --- |
| implementation baseline lock | closed | stable architecture host、internal substrate posture 与 stream/handoff objects 已落地 |
| parser / authored grammar | closed | canonical envelope、action headers、side-specific omission tokens 与 horizontal truncation rejection 已稳定 |
| selection resolution | closed | deterministic contiguous denotation、source/target mismatch blame 与 authored-line diagnostics 已稳定 |
| shared substrate extraction | closed | `mutation_support.rs` 已承载 path identity、normalization、affected-path merge，并对 splice runtime 复用 |
| runtime core | closed | full action basis、alias-aware connected-unit grouping、original-snapshot planning、partial-success accounting 与 targeted newline-boundary normalization 已闭合 |
| CLI / MCP surface | closed | CLI/MCP parity、splice-source-aware diagnostics 与当前 outer-surface contract 已落地 |
| diagnostics family hardening | closed | outer surface 已切换到 structured `error[SPLICE_*]: ...` contract |
| execution accounting / release-grade QA closure | closed | cargo test、CLI/MCP parity 与 docs truthfulness 已全部通过 |

## Evidence Snapshot

- `docutouch-core/src/splice_program.rs`
- `docutouch-core/src/splice_selection.rs`
- `docutouch-core/src/splice_runtime.rs`
- `codex-apply-patch/src/mutation_support.rs`
- `docutouch-server/src/splice_adapter.rs`
- `docutouch-server/src/cli.rs`
- `docutouch-server/src/tool_service.rs`
- `docutouch-server/tool_docs/apply_splice.md`

## Verification Basis

- parser / selection / runtime tests exist under `docutouch-core/tests/`
- CLI / MCP parity and splice failure/source-path presentation exist under `docutouch-server/tests/` and `docutouch-server/src/splice_adapter.rs`
- full workspace verification currently passes via `cargo test --target-dir C:\Users\t103o\.codex\memories\splice-build-20260324c`

## Boundary

本页只记录 implementation wave 的 actual standing，
不重写 stable spec、implementation plan 或 readiness gate 本体。
