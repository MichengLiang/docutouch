(records-disposition-contract)=
# 30 Disposition 作者契约

## 契约范围

本页裁定哪些最终处置结论应进入 `disposition/`。

## Allowed Objects

- rejection record
- superseded record
- absorbed-into record
- final disposition note

## Disallowed Objects

- raw inventory item
- migration-only routing note
- accepted object正文

## Dependency Discipline

- disposition 页必须说明 disposition action、触发对象与去向；
- 若存在 successor，应显式回指；
- 若对象的 canonical identity 已退出当前现行面，应优先由 disposition 裁定其命运，而不是继续停留在 change 或 migration 的语义里；
- 若历史路径仍承担解释、审计或防御负荷，应在记录中注明值得被 knowledge 选择性回指。
