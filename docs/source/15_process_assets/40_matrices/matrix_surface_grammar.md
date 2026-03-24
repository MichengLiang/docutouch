(process-assets-matrix-surface-grammar)=
# Matrix Surface Grammar

## Role

本页定义 execution-facing matrix 的最小字段集。

## Core Judgment

当前路线下，matrix 的正式表达法采用：

- fixed prose preface
- typed matrix table
- 必要时的极短 note

## Typed Matrix Table

当前接受的最小字段集为：

```text
Relation Type | Source | Target | Status | Note
```

当前推荐的 `Status` 工作性取值至少包括：

- `planned`
- `active`
- `blocked`
- `satisfied`
- `closed`

### `Relation Type`

表达当前矩阵陈述的关系类型。

### `Source`

表达关系的起点对象。

### `Target`

表达关系的目标对象。

### `Status`

表达该关系在当前 matrix 中的 standing。

### `Note`

只承接必要的极短限定说明。

## Required Preface

matrix page 在表格前，
至少应以极短 prose 说明：

- 当前矩阵的作用域
- 当前矩阵覆盖的对象类型
- 当前矩阵不覆盖什么

## Boundary

matrix surface 不应退化为：

- 长篇 narrative plan；
- 无字段的随意表格；
- 只靠 prose 暗示的责任关系。
