(meta-first-class-object-and-page-folder-criteria)=
# First-class 对象与 Page / Folder 判据

## 角色

本页定义：

- 什么对象值得 first-class；
- 什么对象应成为 folder-level host；
- 什么对象应保持为单页；
- 什么对象只应停留在 section。

它处理的是结构判据，不处理文风偏好。

这里所说的“First-class 对象”，
指值得被明确 materialize 为独立 page、独立 folder 或独立结构单位的对象，
而不是“看起来重要”的对象。

## First-class 判据

```{list-table}
:header-rows: 1

* - 判据
  - 要问的问题
  - 若不成立的处理
* - Semantic Load
  - 是否承担不可替代的语义边界
  - 不成立则不应 first-class
* - Addressability
  - 是否需要稳定引用、稳定 cross-reference、稳定 review target
  - 不成立则优先降到 section
* - Maintenance Locality
  - 是否有清楚自然宿主
  - 不成立则不应急于新建对象
* - Reuse Value
  - 是否跨多个对象反复出现
  - 只服务单次写作便利则不应提升
* - Query / Validation Benefit
  - 是否显著提升分类、检索、校验、治理
  - 无结构收益则不应提升
* - Temporal Stability
  - 是否超出单轮过程资产的临时需要
  - 不成立则优先保留为 process asset 或 source material
```

## Folder-level Host 的判据

一个对象只有在同时满足以下条件时，才应成为 folder-level host：

1. 它本身是成员宿主位；
2. 它有明确成员资格边界；
3. 它需要局部 charter；
4. 它需要局部 operation rule；
5. 它内部成员之间不是一个 page 的 section 关系，而是真正的成员关系。

若上述条件不成立，则不得仅因“很重要”或“被大量引用”而提升为 folder。

## Atomic Page 的判据

一个对象在以下情况下应保持为单页：

1. 它是完整可引用对象；
2. 它不承载成员资格；
3. 它不需要局部 admission rule；
4. 它的边界足以在单页内完整表达。

高 authority value、高引用频率、高 review 压力，都不自动推翻这四条。

## Section 的判据

若一个内容：

- 没有独立地址收益；
- 没有独立 maintenance locality；
- 没有独立 query / validation 收益；
- 只是某篇文章内部的局部结构；

则应停留在 section，而不是提前提升为 page 或 folder。

## 禁止使用的错误判据

以下说法不构成合法的结构判据：

- “以后可能还会继续长”；
- “以后可能塞更多东西”；
- “它很重要，所以给它一个目录”；
- “很多地方会引用它，所以给它一个目录”；
- “平铺着看起来有点多，所以给它包一层目录”。

这些都不是成员宿主位的判据。

## Page / Folder / Section 的晋退规则

### Page -> Folder

只有当对象身份发生变化，
从原子对象变为成员宿主位时，
才允许 page -> folder。

### Folder -> Page

若某个容器不再承载成员资格，
其局部 charter 与局部 contract 已无独立职责，
则应评估是否退回单页。

### Section -> Page

只有当独立地址、独立维护与独立校验收益成立时，
局部 section 才值得提升为 page。

## 结论

当前仓库中，folder 不是“重要对象的更大实现”；
folder 是容器对象的实现。

若这一条不被写成本地 authority，
后续所有关于目录、页面和 support surface 的争论都无法独立裁定。
