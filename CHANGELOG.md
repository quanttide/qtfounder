# Changelog

## [Unreleased]

### Added

- CLI `code` 子命令：代码语用检查四层（词汇／句法／语篇／语用）。解析与判定分开——判定层只读 `ir::Facts`；Rust 走 `syn`、Python 走 `tree-sitter`，Go／TypeScript／Dart 共用一份通用提取器、差异全在 `specs.rs` 的一份 `LangSpec` 里，加语言只写配置。五语言一致性由测试守住，22 个单元测试。阈值在 `src/cli/data/rules/default-contract.toml`（内嵌，`--rules` 可覆盖）。`unused_imports` 刻意不判——rustc 已权威负责

### Changed

- 数据源同步：主仓库子模块 `assets/memory` → `assets/context` → `data/context`、`quanttide-founder` 组织迁至 `quanttide`；文档路径示例、部署 workflow 的数据源仓库引用与官网链接同步（环境变量名 `QTFOUNDER_MEMORY_PATH` 与运行时目录 `data/memory` 保持不变）

### Removed

- Studio 业务代码（`src/studio/lib/bloc`、`models`、`repositories`、`screens`、`services`、`config.dart`）与测试归档至 `data/archive/platform/qtfounder/studio/`；`src/studio` 仅保留入口 `lib/main.dart` 与最小骨架
- CLI `health` 子命令（check/track/history/profile）：默认路径随 `docs/` → `assets/` 迁移失效、无使用痕迹，情绪画像归 memory 五轴档案（`profile/emotions.md`）维护；`src/cli` 保留入口与结构作为新功能脚手架，API 参考 CLI 页与 README/STATUS/AGENTS/开发指南索引同步移除

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
