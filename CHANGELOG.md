# Changelog

## [Unreleased]

### Changed

- CHANGELOG 拆分：`cli/v0.0.1` 至 `cli/v0.0.3` 三个版本段移至 `src/cli/CHANGELOG.md`，与 `src/site/CHANGELOG.md` 的模块化约定一致；本文件只留仓库级版本，模块版本按 `<模块>/CHANGELOG.md` 维护
- studio 文档按当前结构重写：README 与 ROADMAP 改为骨架 + review_tool 的现状；归档 models/screens 的设计合并到 `doc/data-sources.md` 与 `doc/planned-ui.md`
- 数据源同步：主仓库子模块 `assets/memory` → `assets/context` → `data/context`、`quanttide-founder` 组织迁至 `quanttide`；文档路径示例、部署 workflow 的数据源仓库引用与官网链接同步（环境变量名 `QTFOUNDER_MEMORY_PATH` 与运行时目录 `data/memory` 保持不变）

## Removed

- studio review 代码清空：`packages/review_tool/`、`lib/review/`、`test/review_tool_smoke_test.dart`、`doc/review.md` 移除，`pubspec.yaml` 去掉 `review_tool` 依赖；`src/studio` 只保留入口 `lib/main.dart`，README、ROADMAP 与 `doc/` 同步为骨架现状，等待重新设计
- Studio 业务代码（`src/studio/lib/bloc`、`models`、`repositories`、`screens`、`services`、`config.dart`）与测试归档至 `data/archive/platform/qtfounder/studio/`；`src/studio` 仅保留入口 `lib/main.dart` 与最小骨架
- studio `doc/models/`、`doc/screens/`、`navigation.md`、`router.md` 的过期设计文档；内容合并到 `doc/data-sources.md` 与 `doc/planned-ui.md`

## [0.1.0] - 2026-08-14

结构类变更：仓库从纯官网仓库重构为"官网（`src/site`）+ CLI 工具（`src/cli`）"双模块结构。

### Added

- CLI 工具 `qtfounder-cli`：`health` 子命令（check / track / history / profile），基于 git commit 历史与 LLM 情绪维度抽取
- OSS 部署工作流（deploy.yml，IaC for qtfounder-site）
- CLI 路线图（`src/cli/ROADMAP.md`）
- 官网设计文档重写（`src/site/docs/index.md`）：从"作品集"转向"创作现场"——创作流时间线、创作域矩阵、改稿轨迹

### Changed

- 仓库结构：所有代码移入 `src/site`，新增 `src/cli`（site/v0.0.1 起）
- `quanttide-agent` 依赖从本地路径改为 crates.io（v0.1.1）
- 官网 CHANGELOG 移至 `src/site/CHANGELOG.md`，并移除过时 v0.0.2 条目

### Removed

- 官网 CHANGELOG 中的 v0.0.2 条目（历史归位到本文件）

## [0.0.2] - 2026-06-27

### Added

- 第二大脑介绍页面（/brain）
- React Router 路由配置

## [0.0.1] - 2025-07-13

### Added

- 首页：Hero、关于、最近作品展示
- 作品列表页 `/works`：全部/改稿筛选
- 作品详情页 `/works/fiction/drafts/:slug`：展示改稿原文
- 路由系统（react-router-dom）
- 14 篇职场言情改稿作品上线
