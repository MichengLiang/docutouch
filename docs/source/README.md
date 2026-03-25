# 关于 `docs/source`

这份说明只解释一件事：为什么当前工作区同时保留 `docs/source/` 与 `docs/guide/`，以及这两层各自承担什么职责。

它不是 `index.md`，不承担目录导航职责，也不取代任何正式 contract page。

## 1. `docs/source/` 是什么

`docs/source/` 是当前项目文档的 source corpus。

这里保留的是作者在长期维护中真正依赖的对象：

- 当前可依赖知识
- 过程性执行对象
- 尚未收敛的 deliberation 对象
- records 与 migration / audit / status 对象

这些对象的写法首先服务于：

- 精确表达
- 稳定引用
- 持续维护
- 与模型协作时的低歧义读取

因此，`docs/source/` 不是为了把项目讲得“顺口”而组织出来的；它首先是作者工作台的一部分。

## 2. 为什么不把 `docs/source/` 直接改写成公开说明书

原因很简单：source corpus 与公开说明书承担的职责不同。

如果把 source corpus 直接改写成 reader-facing prose，会同时损失两边：

- source corpus 会被引导性文案、重复解释、读者安抚性 prose 污染；
- 公开说明书又会被过高密度的 source-side 术语、records、process objects 与 deliberation 对象拖慢。

当前工作区不打算用一份正文同时承担这两类职责。

## 3. 为什么额外增加 `docs/guide/`

`docs/guide/` 是当前工作区为公开仓库新增的投影层。

它做的事情很有限：

- 介绍项目是什么
- 说明怎么开始使用
- 用正常仓库文案解释核心工具
- 给公开读者提供稳定入口

它不负责定义权威事实，也不接管 source corpus 的对象边界。

换句话说：

- `docs/source/` 保留 source truth 与 authoring utility
- `docs/guide/` 提供 public explanation 与 reading entry

这不是两套互相竞争的正文，而是一套 authoritative corpus 配一层公开投影。

## 4. 投影层与 source corpus 的关系

`docs/guide/` 可以：

- 调整信息顺序
- 压缩细节密度
- 选出公开读者首先需要知道的对象
- 用更普通的开源仓库文体重写介绍方式

`docs/guide/` 不应：

- 静默改写 source corpus 已确定的事实
- 发明 source corpus 中不存在的行为
- 反向要求 source corpus 为公开可读性而变形

因此，guide 是 projection，不是 replacement。

## 5. 为什么 `docs/source/` 里会同时出现 knowledge / process / deliberation / records

因为当前项目并不只维护“现行结论”，还维护：

- 过程资产
- 未收敛对象
- 审计与迁移记录

如果这些对象不进入同一 source corpus，长期维护时就会重新退回聊天记录、临时笔记或不可搜索的散落状态。

当前组织方式的目标不是让初次读者轻松扫完，而是让作者在长期演化中始终知道：

- 什么已经成立
- 什么仍在讨论
- 什么只是过程对象
- 什么只是历史记录

## 6. 当前仓库里的阅读习惯

对公开读者：

- 从 `docs/README.md` 与 `docs/guide/` 进入

对作者与模型协作：

- 默认把 `docs/source/` 视为当前项目文档的权威依据
- 但仍然按任务选择读取范围，不把整棵树无差别塞进上下文

这意味着：

- `docs/source/` 很重要
- 但“重要”不等于“每次都要全读”

## 7. 这份说明存在的理由

当前仓库保留 `docs/source/`，同时新增 `docs/guide/`，对第一次进入工作区的人来说会自然产生一个问题：

- 为什么不只留一套？

这份 README 的职责就是回答这个问题。

它回答的是结构关系，而不是具体产品行为；
具体产品行为仍然应回到：

- `docs/guide/`
- 对应 crate README
- `docs/source/` 内的正式 interface / architecture / decision / operations pages
