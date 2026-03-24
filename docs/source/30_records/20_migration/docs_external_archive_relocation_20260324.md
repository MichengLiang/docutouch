(records-migration-docs-external-archive-relocation-20260324)=
# Docs External Archive Relocation 2026-03-24

## Role

本页记录 2026-03-24 对 `docs/source/` 外 Markdown 原件执行的 archive relocation。

它回答：

- 哪些原件被移出工作注意力主路径；
- archive root 在哪里；
- 归档后如何从原路径机械追溯到 archive 中的实际文件；
- 为什么这次物理迁移不等于声称所有对象都已升格为 accepted knowledge。

## Source Reality

归档前：

- `docs/source/` 外仍保留一批根部与 `docs/temporary/` 下的 Markdown 原件；
- 本次 archive wave 实际移动了 46 份 Markdown 原件；
- 这些原件已经被 {ref}`records-migration-docs-markdown-ledger` 裁定 object-domain、target host 与 action；
- 但它们继续停留在主工作区，会反复占用注意力并制造“它们是否仍是 canonical host”的视觉噪声。

## Archive Root

- `docs/archive2026年3月24日/`

## Relocation Rule

本次 archive relocation 保留 `docs/` 下的相对路径。

也就是说：

- `docs/product_positioning.md` -> `docs/archive2026年3月24日/product_positioning.md`
- `docs/roadmap.md` -> `docs/archive2026年3月24日/roadmap.md`
- `docs/temporary/foo.md` -> `docs/archive2026年3月24日/temporary/foo.md`

因此，只要已知某个原始 `Source Artifact` 的旧路径，就能机械推出其 archive 后的位置。

## Migration Judgment

这次迁移的判断是：

- 它是对 source material 的物理归档，而不是对对象语义的重新裁决；
- 它不声称所有原件都已被提升为 accepted knowledge；
- 它只依赖一个前提：每个原件已经在 {ref}`records-migration-docs-markdown-ledger` 中拥有明确的 target host 或 disposition judgment；
- 归档之后，current canonical host 继续由 `docs/source/` 内的 accepted / process / deliberation / records objects 承担。

## Scope

本次归档覆盖：

- `docs/source/` 外的 Markdown 原件；
- 不覆盖 `docs/source/` 当前 canonical pages；
- 不覆盖 build 输出与 build asset。

## Outcome

- archive relocation completed on 2026-03-24;
- archive root currently preserves the moved originals in relative-path-stable form;
- `docs/source/` 继续承担 current canonical host。

## Backlinks

- source judgment ledger: {ref}`records-migration-docs-markdown-ledger`
- current migration family host: {ref}`records-migration-index`
