(records-migration-contract)=
# 20 Migration 作者契约

## 契约范围

本页裁定哪些迁移动作与迁移去向应进入 `migration/`。

## Allowed Objects

- migration record
- relocation note
- absorption mapping
- split / merge migration note

## Disallowed Objects

- inventory item
- final disposition-only note
- accepted object正文

## Dependency Discipline

- migration 页应显式指向 source object 与 target host；
- migration 页不能只说“迁了”，必须说明迁到哪里；
- absorb、rewrite、relocate 若仍以 source-to-target mapping 为主问题，应优先进入 `migration/`
- 若对象主要在说明“最终判决”，应迁入 `30_disposition/`。
