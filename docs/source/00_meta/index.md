(meta-index)=
# 00 Meta

## 作用域

本目录承载关于整套文档体系自身的已接纳元知识。

它回答的是：

- 我们为什么这样组织；
- 借了哪些学术与标准资源；
- 树状结构与横切分面分别是什么；
- 文件夹、页面与 section 分别在何种条件下承担边界职责；
- 容器 surface 与成员 surface 如何区分；
- 哪些对象属于 source authority，哪些只是 process asset 或 source material；
- MyST / Sphinx 在这套体系里承担什么角色；
- 页面如何在 `knowledge`、`deliberation`、`records` 三个正文对象域之间迁移；
- canonical source、template-role overlay 与 exported template 如何分属不同 standing。

## 本目录不是什么

- 不是根级维护总契约；
- 不是具体项目知识正文；
- 不是框架尚未收敛时的工作台；
- 不是迁移与审计记录仓。

未收敛的框架问题应进入 `20_deliberation/`；
框架变更的迁移与处置记录应进入 `30_records/`。

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向 `00_meta/` 容器本身的 operation surface
* - `10_framework_scope.md`
  - Source-bearing article
  - 定义体系范围与非目标
* - `20_methodology_commitments.md`
  - Source-bearing article
  - 定义理论采用与排除边界
* - `30_taxonomy_and_facets.md`
  - Source-bearing article
  - 定义 object-domain taxonomy 与横切分面
* - `40_theory_mapping.md`
  - Source-bearing article
  - 定义理论映射与精度标记
* - `50_writing_and_citation.md`
  - Source-bearing article
  - 定义语义职责与表达面的关系
* - `60_promotion_and_disposition_policy.md`
  - Source-bearing article
  - 定义三域内部对象迁移规则
* - `70_glossary.md`
  - Source-bearing article
  - 定义稳定术语
* - `80_boundary_types_and_container_semantics.md`
  - Source-bearing article
  - 定义边界类型与容器语义
* - `90_surface_roles_and_object_kinds.md`
  - Source-bearing article
  - 定义 object kinds 与 surface roles
* - `100_first_class_object_and_page_folder_criteria.md`
  - Source-bearing article
  - 定义 page / folder / section 的判据
* - `110_self_containment_and_external_grounding_policy.md`
  - Source-bearing article
  - 定义本地 authority 与外部 grounding 的边界
* - `120_process_assets_and_authority_conversion_policy.md`
  - Source-bearing article
  - 定义外围过程资产向 authority 的转换规则
* - `130_build_root_and_authority_role_distinction.md`
  - Source-bearing article
  - 定义构建根与 authority role 的正交区分
* - `140_canonical_source_and_template_export_policy.md`
  - Source-bearing article
  - 定义单一真源、overlay、contract 下沉与模板导出的 authority 边界
```

## 阅读顺序

1. {ref}`meta-framework-scope`
2. {ref}`meta-methodology`
3. {ref}`meta-taxonomy`
4. {ref}`meta-boundary-types-and-container-semantics`
5. {ref}`meta-surface-roles-and-object-kinds`
6. {ref}`meta-first-class-object-and-page-folder-criteria`
7. {ref}`meta-theory-mapping`
8. {ref}`meta-writing-and-citation`
9. {ref}`meta-self-containment-and-external-grounding`
10. {ref}`meta-process-assets-and-authority-conversion`
11. {ref}`meta-build-root-and-authority-role-distinction`
12. {ref}`meta-canonical-source-and-template-export`
13. {ref}`meta-promotion-policy`
14. {ref}`meta-glossary`

## 页面目录

```{toctree}
:maxdepth: 2

authoring_contract
10_framework_scope
20_methodology_commitments
30_taxonomy_and_facets
80_boundary_types_and_container_semantics
90_surface_roles_and_object_kinds
100_first_class_object_and_page_folder_criteria
40_theory_mapping
50_writing_and_citation
110_self_containment_and_external_grounding_policy
120_process_assets_and_authority_conversion_policy
130_build_root_and_authority_role_distinction
140_canonical_source_and_template_export_policy
60_promotion_and_disposition_policy
70_glossary
```
