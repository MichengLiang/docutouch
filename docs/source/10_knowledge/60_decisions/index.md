(knowledge-decisions-index)=
# 60 Decisions

## 作用域

本目录承载 accepted decision family 与其必要理据。

它回答：哪些具体裁决已经被接受，以及这些裁决围绕什么对象成立。

## 典型对象

- accepted ADR
- accepted decision record
- accepted QOC outcome
- accepted rationale package
- consequence summary
- alternatives summary

## 不承担的职责

- 不写 open issue
- 不写 unresolved proposal
- 不写高层长期原则
- 不写 architecture description 本体

## Dependency Position

### Upstream Dependencies

- `30_principles/`
- `40_requirements/`
- `50_architecture/`
- `70_interfaces/`
- `80_operations/`（若是操作层 decision）

decision 对象应只链接其真实的 authority basis；不得为了“保险”而默认把所有可能上游都挂上去。

### Downstream Dependents

一般不作为主体家族的 authority 上游。

### Lateral Cross-References

- 主体家族可引用 decisions 作为 accepted rationale
- 这种引用不应升级为“主体家族依赖 decisions 才成立”

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 family 容器本身的 operation surface
* - `apply_patch_warning_first_rationale.md`
  - Source-bearing article
  - 记录 `apply_patch` warning-first compatibility posture 的 accepted rationale
* - `apply_splice_apply_patch_separation_rationale.md`
  - Source-bearing article
  - 记录 `apply_splice` 与 `apply_patch` 必须分立的 accepted rationale
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
apply_patch_warning_first_rationale
apply_splice_apply_patch_separation_rationale
```
