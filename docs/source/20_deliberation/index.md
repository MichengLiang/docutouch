(deliberation-index)=
# 20 Deliberation

## 作用域

本目录承载尚未收敛、不能被下游依赖的论证与设计对象。

它不是垃圾桶，也不是“以后再说”的模糊区，而是未收敛对象的正式宿主面。

## 内部结构

`20_deliberation/` 内部不采用一组彼此毫无关系的平行桶，而采用：

- 主干 deliberation chain
- 桥接 / 诊断层
- 派生支撑层

### 主干 deliberation chain

- `issues/`
- `proposals/`
- `candidate_specs/`

### 桥接 / 诊断层

- `assumptions/`
- `conflicts/`
- `evidence_gaps/`

### 派生支撑层

- `worklists/`

## 总体依赖图

```{list-table}
:header-rows: 1

* - 家族
  - 主要上游
  - 主要下游
* - `issues/`
  - accepted knowledge family、必要时 records context
  - `proposals/`、`conflicts/`、`evidence_gaps/`、`worklists/`
* - `proposals/`
  - `issues/`、相关 accepted knowledge family
  - `assumptions/`、`candidate_specs/`、`worklists/`
* - `assumptions/`
  - `issues/`、`proposals/`、相关 accepted knowledge family
  - `candidate_specs/`、`worklists/`
* - `candidate_specs/`
  - `proposals/`、`assumptions/`、相关 accepted knowledge family
  - `10_knowledge/` 对应家族或 `30_records/`
* - `conflicts/`
  - `issues/`、`proposals/`、`candidate_specs/`、相关 accepted knowledge family
  - `proposals/`、`worklists/`、最终 accepted decision
* - `evidence_gaps/`
  - `issues/`、`proposals/`、`assumptions/`、`candidate_specs/`
  - `worklists/` 与相关对象的后续收敛
* - `worklists/`
  - 各 deliberation 家族
  - 一般不作为其他家族的 authority 上游
```

## 二级家族

- `issues/`
- `proposals/`
- `assumptions/`
- `candidate_specs/`
- `conflicts/`
- `evidence_gaps/`
- `worklists/`

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向 `20_deliberation/` 容器本身的 operation surface
* - `10_issues/`、`20_proposals/`、`40_candidate_specs/`
  - Family container
  - 主干 deliberation chain 容器
* - `30_assumptions/`、`50_conflicts/`、`60_evidence_gaps/`
  - Family container
  - 桥接 / 诊断层容器
* - `70_worklists/`
  - Family container
  - 派生支撑层容器
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
10_issues/index
20_proposals/index
30_assumptions/index
40_candidate_specs/index
50_conflicts/index
60_evidence_gaps/index
70_worklists/index
```
