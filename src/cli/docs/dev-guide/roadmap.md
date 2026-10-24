# ROADMAP

量潮创始人平台（qtfounder）路线图。状态报告在仓根的 `STATUS.md`（不进本部署），模块级路线图见 `src/*/ROADMAP.md`。

## CLI v0.1

当前为 `cli/v0.0.7`，已落地 `code`、`language`、`health`、`docs` 四个子命令，`health` 下另有 `reflect`。v0.1 的文档结构审计五项已随 v0.0.3 与 v0.0.5 提前落地，只剩阈值标定。

### 范围

- [x] 新 `docs` 子命令：表格层从 `language` 整层迁出（`table.rs`、`[table]` 契约、`Kind::Table`），`language` 不再保留表格检查
- [x] `docs` 列表层：三段式列表项与超长列表项；契约 `[list]` 段，判定层 `src/docs/list.rs`
- [x] 密度兜底：结构块占比与无散文率，挂在 `docs` 报告里，只出嫌疑；阈值默认关闭
- [x] `docs check`：`.agents/skills/` 实际目录与 `AGENTS.md`、`CONTRIBUTING.md` 的索引比对
- [x] 加粗与分割线：每千字加粗处数、一行并列加粗、整句加粗；分割线总数与紧挨标题的装饰线
- [ ] 阈值标定：两组语料各十五篇，用分位数替换 `[list]` 与密度层的待定值

取法与反样本目录见[文档审计路线图](docs/roadmap.md)「阶段三」。

### 不做

- 自动改写：改写需要判断，属于 AI 的活，工具只出报告
- 多格式：Markdown 没跑通之前不做 docx 与 HTML
- `auto` 推断 Profile：黑盒不可调试，手动 `--rules` 一行参数的事

### 验收

拿真实文档跑一遍，报告贴回 AI，它能按位置改完，复跑达标；`cargo test` 全绿；打 `cli/v0.1.0`。

## 迁移

把当前的 studio 实现移动到 qtcloud 并且清空现有实现，以方便重新定义一个更自由的版本。

### 资产

- [ ] 先封装到 toolkit 仓库，然后迁移到 qtcloud-asset
- [ ] 评估是否以及如何基于 OSS 的资产管理功能合并

### 写作

- [ ] 封装到 toolkit 仓库，然后对比写作云

## 新增

使用 quanttide-founder 的结构迁移到私有第二大脑 iGuo，统一管理公私两个结构。探索未来如何管理组织层面的公开和私有结构。

### 资产结构

- [ ] 公开和私有第二大脑都有 fiction 和 memory，且前者是成品，后者是素材；搭建一个稳定且显式的结构有利于同构维护

### 创意写作

- [ ] fiction 都来源于 memory，且私有的结构还没有完全建立起来；需要一个精细的结构来加工它
