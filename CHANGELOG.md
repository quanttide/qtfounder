# Changelog

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
