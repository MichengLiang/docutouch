(records-migration-process-assets-host-introduction)=
# Process Assets Host Introduction

## Role

本页记录 corpus-level process assets 的 canonical host 引入迁移。

## Source Reality

在当前变更之前：

- execution-facing process objects 已被承认为 object kind；
- 但缺少 dedicated build-root host；
- 相关内容只能分散依附于 `00_meta/` 说明、`70_worklists/` 派生动作面、
  或外围 working notes。

## Target Host

当前 canonical host 为：

- `15_process_assets/`

并细分为：

- `10_exec_plans/`
- `20_work_packages/`
- `30_handoffs/`
- `40_matrices/`
- `50_readiness/`

## Migration Judgment

本次迁移的核心不是“把若干现成页面挪目录”，
而是把原本缺失 canonical host 的 process responsibility，
显式迁入新的 object-domain 宿主位。

## Backlinks

- current canonical host: `15_process_assets/`
- related change record: `30_records/40_change/root_topology_process_assets_change.md`
