(records-migration-index)=
# 20 Migration

## 作用域

本目录承载迁移记录对象。

它回答：某个对象从旧宿主如何进入新宿主，或如何被拆分、吸收、重写到新结构里。

## 典型对象

- migration note
- source-to-target relocation record
- split / merge migration record
- absorbed-into mapping
- rewritten-into mapping

## 不承担的职责

- 不做 inventory
- 不做最终 disposition judgment
- 不做 accepted knowledge 正文

## Dependency Position

### Upstream Dependencies

- `10_inventory/`
- relevant accepted / unresolved object

### Downstream Dependents

- `30_disposition/`
- `60_status/`
- `70_coverage/`
- current canonical host

### Lateral Cross-References

- `40_change/` 可与 migration 并存，但不应被混为一页

## Exit / Refresh Logic

migration 记录本身保留；
它应尽量指向新宿主、successor 或吸收目标，而不是把读者留在旧对象上。

当对象的主要问题是“从哪里迁到哪里、如何被吸收或重写”时，
应优先进入 `migration/`；
若主要问题已经变成“最后怎样判”，则应 hand-off 到 `disposition/`。

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 family 容器本身的 operation surface
* - `process_assets_host_introduction.md`
  - Actual record object
  - 记录 corpus-level process-assets host 的引入迁移
* - `docs_markdown_migration_ledger.md`
  - Actual record object
  - 记录 `docs/` 根部与 `docs/temporary/` 当前迁移裁决
* - `docs_external_archive_relocation_20260324.md`
  - Actual record object
  - 记录 `docs/source/` 外 Markdown 原件归档到 archive root 的物理迁移
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
process_assets_host_introduction
docs_markdown_migration_ledger
docs_external_archive_relocation_20260324
```
