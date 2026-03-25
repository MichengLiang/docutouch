(deliberation-candidate-specs-index)=
# 40 Candidate Specs

## 作用域

本目录承载已经接近规格形态、但尚未进入 accepted knowledge 的候选对象。

它回答：哪些对象已经长成 specification surface，但仍未完成接纳。

candidate spec 是 accepted promotion 的默认直接前置态，
但它自身仍停留在 deliberation object-domain，
不得提前承担现行真值。

## 典型对象

- candidate requirement set
- candidate architecture description
- candidate interface contract
- candidate operations spec
- candidate structured policy text

## 不承担的职责

- 不写普通 proposal
- 不写 accepted specification
- 不写 generic issue
- 不写 worklist

## Dependency Position

### Upstream Dependencies

- `20_proposals/`
- `30_assumptions/`
- relevant accepted knowledge family

### Downstream Dependents

- `10_knowledge/` 对应主体家族
- `30_records/`（若被处置）

### Lateral Cross-References

- `50_conflicts/` 可标出 candidate spec 之间的冲突
- `60_evidence_gaps/` 可标出 candidate spec 尚缺的担保

## Exit Routes

- 若被接纳，应在 `10_knowledge/` 对应家族建立 canonical host，并将原页改写为显式指针或处置说明
- 若被放弃，迁入 `30_records/disposition/`
- 若被拆解回更早阶段，回流到 `proposals/` 或 `issues/`

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 family 容器本身的 operation surface
* - `apply_splice_formal_semantics_draft.md`
  - Process / Deliberation Object
  - 记录 `apply_splice` formal grammar 与 execution semantics 的 candidate spec
* - `apply_patch_line_number_assisted_locking_draft.md`
  - Process / Deliberation Object
  - 保留 `apply_patch` line-number-assisted locking promotion 前的 draft 摘要与 pointer
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
apply_splice_formal_semantics_draft
apply_patch_line_number_assisted_locking_draft
```
