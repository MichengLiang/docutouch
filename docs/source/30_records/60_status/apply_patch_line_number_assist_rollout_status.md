(records-status-apply-patch-line-number-assist-rollout)=
# Apply Patch Line-Number-Assist Rollout Status

## Role

本页记录 `apply_patch` line-number-assisted locking 从 baseline authority lock 到 code/doc/test closeout 的实际执行状态。

## Current Standing

| Work Item | Current Standing | Note |
| --- | --- | --- |
| baseline authority lock | closed | accepted rationale、candidate draft、rollout plan、acceptance gate、work package 与 stream matrix 已落地 |
| parser support | closed | numbered `@@` header 继续由 parser 识别；body-level `N | text` 默认保持 parser-neutral，并保留 truthful authored blame |
| runtime semantics | closed | default `header_only` / advanced `full` mode split 已落地；header-level numbering 默认生效，dense body numbering 仅在 `full` mode 下解释，并以 original snapshot 解释；另有窄 repeated-first-old-line compatibility path 用于接住高频 LLM-authored numbered-header shape |
| presentation / tool-doc sync | closed | injected tool docs、outer visible examples、example showcase 与 stable interface wording 已对齐“optional numbered assist + hidden advanced mode”；兼容面新增 repeated-first-old-line 事实性披露，但不改 canonical public guidance |
| CLI / MCP parity | closed | CLI/MCP smoke paths 已覆盖 numbered-assist success、failure、warning 与 path-source recovery |
| env / CLI mode control | closed | process env default 与 CLI 单次 override 已闭合，MCP 仍只受 process env 影响 |
| regression closure | closed | workspace `cargo test` 已通过，feature 与 mode refinement 均不再停留在 proposal-only 状态 |

## Evidence Snapshot

- `codex-apply-patch/src/parser.rs`
- `codex-apply-patch/src/lib.rs`
- `codex-apply-patch/tests/suite/cli.rs`
- `codex-apply-patch/tests/suite/tool.rs`
- `docutouch-core/src/patch_runtime.rs`
- `docutouch-core/src/patch_presentation.rs`
- `docutouch-server/src/cli.rs`
- `docutouch-server/src/patch_adapter.rs`
- `docutouch-server/tests/cli_smoke.rs`
- `docutouch-server/tests/stdio_smoke.rs`
- `docutouch-server/tool_docs/apply_patch.md`
- `codex-apply-patch/apply_patch_tool_instructions.md`
- `docs/source/10_knowledge/70_interfaces/apply_patch_semantics.md`

## Verification Basis

- full workspace verification currently passes via `cargo test`
- example script remains syntactically valid via `uv run python -m py_compile example/1.py`
- representative numbered-assist success/failure paths are covered at parser, vendored CLI, DocuTouch CLI, and MCP layers

## Boundary

本页只记录 rollout wave 的 actual standing，
不重写 accepted interface semantics、accepted rationale 或 future reopened proposals 本体。
