(meta-glossary)=
# 术语表

## 说明

本术语表收整套体系的稳定工作术语。

若某术语尚未稳定，不应先收入此处，而应在 `20_deliberation/` 中继续收敛。

```{glossary}
元知识子树
    承载关于整套体系自身之范围、方法、分类法、理论映射、表达规则与状态变化规则的已接纳知识子树。当前对应 `00_meta/`。

作者契约
    裁定某个目录如何被继续维护、何类对象可以进入、何类对象必须迁出，以及对象如何在不同对象域之间移动的规则页。当前默认文件名为 `authoring_contract.md`。

当前可依赖知识
    已被接纳、可被下游实现、评审、推理或引用依赖的知识对象。当前宿主对象域为 `10_knowledge/`。

未收敛对象
    尚未完成必要担保、因而不能被下游依赖的论证与设计对象。当前宿主对象域为 `20_deliberation/`。

记录对象
    主要记录变化、迁移、处置、覆盖、审计与状态事实，而不承担现行真值的对象。当前宿主对象域为 `30_records/`。

高层原则
    能持续约束后续多项局部决策、且已经进入当前可依赖知识面的原则性对象。当前默认宿主为 `10_knowledge/30_principles/`。

accepted decision
    已经完成裁决并进入当前可依赖知识面的决策对象。当前默认宿主为 `10_knowledge/60_decisions/`。

candidate specification
    仍处于未收敛状态、尚未正式进入当前可依赖知识面的规格对象。计划中的默认宿主为 `20_deliberation/40_candidate_specs/`。

升格
    一个对象从 `deliberation` 进入 `knowledge`，成为可被下游依赖对象的状态变化。

降格
    一个对象退出 `knowledge` 的现行地位，不再作为当前可依赖知识保留的状态变化。

处置
    对象在不再承担现行职责后，被保留、替代、删除、归档或仅以记录形态继续存在的决定。

映射精度
    在 `theory_mapping` 页面中用于说明外部理论与本体系局部规则之间关系强度的工作性标签。

构建根
    承担 Sphinx / MyST 构建入口的物理宿主位。它回答页面是否进入当前可构建语料，不直接决定对象的 authority role。当前对应 `temporary/docs/source/`。

物理边界
    由文件夹、文件、路径等物理实现提供的边界。它回答对象被放在哪里，不自动回答该边界是否已具备语义成员资格。

操作边界
    回答什么能进入、什么时候 hand-off、谁维护的边界。当前常由 `authoring_contract.md` 承担。

语义边界
    回答哪些对象属于这里、哪些不属于这里的边界。一个对象可以具备强语义边界，却仍然只是原子 page，而不是容器。

容器对象
    承载成员资格、局部 charter、局部 operation rule 的对象。当前默认通过“文件夹 + `index.md` + `authoring_contract.md`”实现。

原子对象页
    自身就是完整可引用对象、但不承载成员资格的页面。高引用度与高 authority value 不自动使其成为容器。

容器 surface
    直接指向容器对象本身的 surface。当前默认包括 `index.md` 与 `authoring_contract.md`。

成员 surface
    指向容器中某个成员对象的 surface，而不是容器本身。

source-bearing article
    承担定义、边界、理据、约束等正式正文职责的成员页。

exported support surface
    承担 vocabulary、relation、schema、status rule、targeting rule、support boundary 等支撑职责的成员页。

source material
    提供 antecedent、证据或历史来源，但不直接承担当前 authority 的材料。

过程资产
    承担 planning、readiness、matrix、discussion coordination 等过程职责的对象。它们可以进入构建根并成为可构建对象，也可以只作为外围工作底稿存在；二者都不自动使其成为 `source authority`。

过程资产子树
    承载 build-root resident process assets 的对象域。当前对应 `15_process_assets/`。

execution plan
    承担复杂事项总体推进、工期安排、并行策略、验收策略与 replan 触发条件的过程对象。当前默认宿主为 `15_process_assets/10_exec_plans/`。

work package
    从 execution plan 拆出的可执行工作包，承接输入、输出、依赖、owner type 与 exit route。当前默认宿主为 `15_process_assets/20_work_packages/`。

agent handoff
    面向单个 executor 的执行 brief，承接允许编辑面、禁改区域、交付物与 verification criteria。当前默认宿主为 `15_process_assets/30_handoffs/`。

task matrix
    承担 task-to-file、ownership、dependency、agent assignment 等 execution-facing relation 的矩阵对象。当前默认宿主为 `15_process_assets/40_matrices/`。

readiness plan
    承担某个 gate 或 rollout 前的准备面对象。当前默认宿主为 `15_process_assets/50_readiness/`。

records authority article
    裁定某类 records object 的 scope、boundary、gate 或 refresh rule 的页面。

actual record object
    记录某次具体迁移、处置、变更、审查或覆盖事实的对象。

self-contained authority
    在不持续外包给外部文档的前提下，足以独立裁定本地结构分类、placement 与转换规则的本地 authority 状态。

一等对象
    值得被明确 materialize 为独立 page、独立 folder 或其他独立结构单位的对象。其成立依据不在于“看起来重要”，而在于语义负荷、可寻址性、维护局部性、复用价值与校验收益等判据。

authority 角色
    某个对象在 authority 结构中承担的角色，例如 source authority、projection、derived view、support surface、source material 或 process asset。

workflow 角色
    某个对象在工作流中承担的角色，例如 domain statement、process artifact、operational guide、reader-facing package。

charter surface
    直接声明某个容器是什么、成员是什么、如何读取的容器 surface。当前默认由 `index.md` 承担。

operation surface
    直接声明某个容器如何维护、成员如何进入与迁出的容器 surface。当前默认由 `authoring_contract.md` 承担。

上游依赖
    指某个对象家族在 authoring 或 authority 上以前置家族为基础的关系。它应尽量保持单向、稀疏、低回路。

侧向交叉引用
    指对象家族之间的普通站内指认关系。它不自动构成 authority dependency。

派生支撑面
    主要汇总、索引、辅助组织其他主体家族对象的页面或家族。它不应反向统治主体对象的 authority。

successor
    在 records 语境中，指被替代、被吸收或被迁移对象的当前后继对象或后继宿主。

current canonical host
    当前被承认为某类对象正式宿主的目录或页面位置。records 应尽量显式回指它。

backlink
    从 records 指向 current canonical host、successor 或处置触发对象的显式回指；在必要时，knowledge 也可以选择性回指 records。

对象级记录
    直接记录某个具体对象的迁移、处置、变更或审查结果的 records 家族成员，例如 `migration/`、`disposition/`、`change/`、`audit/`。

聚合记录
    对多个对象级记录进行汇总、监控或覆盖统计的 records 家族成员，例如 `status/` 与 `coverage/`。
```
