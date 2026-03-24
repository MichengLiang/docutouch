(records-status-apply-splice-engineering-hardening)=
# Apply Splice Engineering Hardening Status

## Role

本页记录 `apply_splice` engineering hardening wave 的执行状态与当前 closeout 结果。

## Source Basis

- {ref}`process-assets-apply-splice-engineering-hardening-plan`
- {ref}`process-assets-apply-splice-engineering-hardening-readiness`
- `30_records/60_status/apply_splice_implementation_status.md`
- current engineering-quality investigation and resulting hardening changes

## Repair Scope

本波次处理的 hardening set 包括：

1. create canonical plan/readiness hosts for the engineering-quality wave
2. align first-entry docs with the accepted internal-substrate posture
3. reduce splice-internal selection-authority duplication
4. reduce generic presentation-helper duplication across patch/splice
5. reduce generic transport-shell duplication across patch/splice CLI flows and adapters
6. add direct regression coverage for the changed seams and rerun the workspace tests

## Current State

| Gate Item | Current Standing | Note |
| --- | --- | --- |
| plan/readiness hosts landed | closed | `15_process_assets/` 已有 canonical engineering hardening hosts |
| doctrine propagation into first-entry docs | closed | root README、server README 与 CLI architecture wording 已补齐 internal-substrate posture |
| splice selection-authority seam | closed | text-level 与 byte-level selection resolution 现在共享一套 authority helper |
| generic presentation-helper duplication | closed | `presentation_shared.rs` 已承载共享展示机械层，splice presentation 也获得直接回归测试 |
| transport-shell duplication | closed | `transport_shell.rs` 已承载 patch/splice 共用的 shell 语义，CLI file/stdin helpers 已统一 |
| workspace verification | closed | `cargo test` 已在工作区根完成通过 |
| deeper substrate extraction | closed | runtime path resolution、affected-path diff、connected-unit grouping 与 generic filesystem transaction mechanics 已共享；splice-specific failed-unit `committed` reporting 仍保留在 tool-owned layer |

## Closed Outcomes

- `codex-apply-patch` 作为 internal substrate 的 accepted posture 已传播到第一入口文档，而不再主要停留在 splice-specific architecture prose；
- splice selection semantics 不再由 `splice_selection.rs` 与 `splice_runtime.rs` 各自维护两套解析/校验 authority；
- patch/splice 现在共享一套 runtime path anchoring / normalization helper，而不再各自包装同构的 path-resolution mechanical layer；
- patch/splice 现在共享 affected-path diff classifier、connected-unit grouping helper 与 generic byte-level filesystem transaction mechanics；
- patch/splice presentation 的共享机械层已抽出，splice 也补上了直接 presentation regression；
- patch/splice transport shell 不再各自维护一份几乎平行的 CLI/path/source shell；
- MCP-side `apply_splice` 现在与 `apply_patch` 一样接受无 workspace 的 absolute-only path 场景；
- 当前工作区测试矩阵可以重新证明本波次后的代码仍然保持 green。

## Residual Note

- splice failed-unit `committed` reporting 仍保持 local，因为它暴露的是 splice-specific partial residue，而不是 generic filesystem transaction mechanics；
- `tool_service.rs` 仍保留 patch/splice 各自的 MCP entry flow，但这现在只是内部 wiring 分工，不再对应不同的路径 contract；
- patch/splice target-anchor wording 仍保持各自的 public surface，而没有强行合并成同一种 renderer。

## Boundary

本页是 status record。

它不承担：

- accepted architecture doctrine 正文；
- future deeper substrate extraction 的总计划本体；
- audit finding 本身的完整论证过程。
