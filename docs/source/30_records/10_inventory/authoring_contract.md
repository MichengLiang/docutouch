(records-inventory-contract)=
# 10 Inventory 作者契约

## 契约范围

本页裁定哪些盘点对象应进入 `inventory/`。

## Allowed Objects

- inventory item
- manifest item
- preliminary classification item
- corpus / subtree inventory page

## Disallowed Objects

- migration record
- disposition note
- accepted knowledge statement
- issue / proposal / candidate spec 正文

## Dependency Discipline

- inventory 负责记录“有什么”，不负责直接裁定“怎么办”；
- 若对象主要在说明去向，应迁入 `20_migration/`；
- 若对象主要在说明最后判决，应迁入 `30_disposition/`。

