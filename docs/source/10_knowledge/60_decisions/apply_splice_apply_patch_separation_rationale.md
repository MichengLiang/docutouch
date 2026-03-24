(knowledge-decisions-apply-splice-apply-patch-separation-rationale)=
# Apply Splice And Apply Patch Separation Rationale

## 作用域

本页记录 `apply_splice` 与 `apply_patch` 必须保持分立的 accepted rationale。

它回答的不是使用技巧，
而是对象边界层面的已接纳裁决：

- 为什么这不是同一工具的两种表面语法；
- 为什么成本或 token 经济性不能替代存在依据；
- 为什么共享底层实现不推出共享产品身份。

## Accepted Decision

当前 accepted decision 是：

- `apply_patch` 与 `apply_splice` 保持为两个独立工具身份；
- 二者的分立依据首先是 object boundary，而不是实现口味或教学便利；
- 后续能力归属应先问“处理的对象是什么”，而不是先问“塞进哪个工具更方便”。

## Accepted Rationale

### Object Boundary

- `apply_patch` 处理的是文本差异；
- `apply_splice` 处理的是现有文本片段的转移关系。

只要这两类对象不同，
它们的工具身份就不应被折叠为一体。

### Authoring Model

- `apply_patch` 允许直接 author 新文本；
- `apply_splice` 只允许声明既有片段与目标位置之间的关系。

这两种 author model 若混入同一表面，
会破坏调用者的心智模型与边界治理。

### Success Contract

- `apply_patch` 的成功围绕差异是否正确建立新文本状态；
- `apply_splice` 的成功还要求 source/target selection、move/copy relation、atomicity 与 overlap legality 成立。

因此，两者并不是同一成功条件下的不同写法。

### Governance Consequence

若不先写清分立依据，
后续需求会持续推动能力膨胀，
最终把两类对象混写进一个边界失守的混合工具。

## Rejected Alternative

当前被拒绝的说法包括：

- 把 `apply_splice` 理解为 `apply_patch` 的另一套 surface syntax
- 仅因结果可模拟，就主张两者应合并
- 用 token 成本或效率优势替代 object-level existence basis

## Operational Consequence

这项裁决导出的协作关系是：

- 当对象是既有片段的复制、移动、删除或替换关系时，优先使用 `apply_splice`
- 当对象已经转为文本修订时，再交给 `apply_patch`

这里的先后顺序反映的是对象变化，
不是工具竞争。

## Source Basis

- `docs/archive2026年3月24日/temporary/为什么 `apply_splice` 与 `apply_patch` 必须分立.md`
- {ref}`knowledge-interfaces-apply-splice-spec`
- {ref}`knowledge-interfaces-apply-patch-semantics`
