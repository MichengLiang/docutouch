(process-assets-index)=
# 15 Process Assets

## 作用域

本目录承载 build-root resident process assets。

这些对象：

- 需要稳定地址与持续维护；
- 需要被目录、搜索、渲染与交叉引用系统发现；
- 不自动承担 accepted truth；
- 不自动等于 actual record。

## 二级家族

- `10_exec_plans/`
- `20_work_packages/`
- `30_handoffs/`
- `40_matrices/`
- `50_readiness/`

## 不承担的职责

- 不写 accepted knowledge 本体
- 不写 issue / proposal / candidate spec 本体
- 不写 actual migration / audit / status record
- 不写 generic scratchpad 或会话内临时思考

## Dependency Position

### Upstream Dependencies

- `00_meta/`
- `10_knowledge/`
- relevant `20_deliberation/`

### Downstream Dependents

- `20_deliberation/70_worklists/`
- `30_records/50_audit/`
- `30_records/60_status/`
- `30_records/70_coverage/`

### Lateral Cross-References

- `30_records/` 可记录 process asset 执行后的事实结果
- `20_deliberation/` 可为 process asset 提供未收敛对象输入

## Member Kinds

```{list-table}
:header-rows: 1

* - 成员
  - Object Kind
  - 说明
* - `authoring_contract.md`
  - Container surface
  - 指向当前 object-domain 容器本身的 operation surface
* - `10_exec_plans/` 至 `50_readiness/`
  - Family container
  - 承载不同 process responsibility 的 family 容器
```

## 页面目录

```{toctree}
:maxdepth: 1

authoring_contract
10_exec_plans/index
20_work_packages/index
30_handoffs/index
40_matrices/index
50_readiness/index
```
