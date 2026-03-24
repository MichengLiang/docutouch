(deliberation-proposals-line-locked-apply-patch-syntax-tradeoff)=
# Line-Locked Apply Patch Syntax Trade-off

## 作用域

本页记录 line-locked `apply_patch` extension 的候选 syntax trade-off。

它回答：

- stronger locking syntax 当前有哪些主要候选；
- 各候选围绕什么 criteria 被比较；
- 当前推荐方向是什么；
- 哪些 syntax path 已被 reject 或 deprioritize。

## Target Accepted Family

- `10_knowledge/70_interfaces/`
- `10_knowledge/60_decisions/`

## Evaluation Criteria

当前 trade-off 统一使用以下 criteria：

1. authoring ergonomics
2. parser complexity
3. ambiguity resistance
4. diagnostics quality
5. token cost
6. host teachability
7. cross-model intuitiveness
8. preservation of patch muscle memory

## Current Candidate Set

### Option A

maximal numbered old-side evidence pattern。

当前判断：

- strongest explicit lock
- strongest diagnostic precision
- 不是 preferred default

### Option B

concise single-anchor form。

当前判断：

- strongest ergonomic default
- 仍不足以覆盖全部 long-tail ambiguity case

### Option C

teach B first, keep A as stronger evidence form。

当前判断：

- 是当前 best direction
- 能把 parser support 与 prompt preference 保持相关但不混同

## Rejected Or Deprioritized Paths

- stacked multiple `@@` anchor hierarchy
- separate `CTX` mechanism as first-class syntax object

## Current Recommendation

当前推荐是 Option C：

- 用 concise anchor form 作为 prompt-facing default；
- 保留 numbered-evidence form 作为 escalation path；
- 不把 D / E 方向继续作为 primary investment target。

这仍是 proposal，
不是 final normative grammar。

## Exit Route

- 若 syntax family 被进一步 formalize，迁入 `20_deliberation/40_candidate_specs/`
- 若 prompt-facing choice 与 rejection set 被 accepted，相关理由进入 `10_knowledge/60_decisions/`
- 若 syntax direction 被放弃，迁入 `30_records/30_disposition/`

## Source Basis

- `docs/archive2026年3月24日/temporary/line_locked_patch_syntax_tradeoff_study.md`
