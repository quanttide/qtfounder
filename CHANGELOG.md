# Changelog

## [Unreleased]

### Added

- studio：新增 `packages/review_tool` 审阅抽样包（扫描/抽样/批次/提交/行级 diff；JSONL 存储，Web 禁用），并在 `lib/review/review_service_factory.dart` 提供接线入口

### Changed

- studio 文档按当前结构重写：README 与 ROADMAP 改为骨架 + review_tool 的现状；归档 models/screens 的设计合并到 `doc/data-sources.md` 与 `doc/planned-ui.md`
- 数据源同步：主仓库子模块 `assets/memory` → `assets/context` → `data/context`、`quanttide-founder` 组织迁至 `quanttide`；文档路径示例、部署 workflow 的数据源仓库引用与官网链接同步（环境变量名 `QTFOUNDER_MEMORY_PATH` 与运行时目录 `data/memory` 保持不变）

### Removed

- Studio 业务代码（`src/studio/lib/bloc`、`models`、`repositories`、`screens`、`services`、`config.dart`）与测试归档至 `data/archive/platform/qtfounder/studio/`；`src/studio` 仅保留入口 `lib/main.dart` 与最小骨架
- studio `doc/models/`、`doc/screens/`、`navigation.md`、`router.md` 的过期设计文档；内容合并到 `doc/data-sources.md` 与 `doc/planned-ui.md`

## [cli/v0.0.2] - 2026-10-22

第三个子命令 `health` 上线：读会话库看消耗落在哪，每条命中挂一句原话。它不做情绪画像。

### Added

- CLI `health` 子命令：读 Hermes 会话库（只读、只取 `role='user'`、同会话同内容去重），
  判两类消耗信号（`fatigue` 真的累 / `friction` 烦与返工）、来源短语与责任环节，
  每条来源必须挂一句原话，没有原话不计命中。口径在 `src/cli/src/health/prompt.md`。
  与 v0.0.1 移除的旧 `health`（check/track/history/profile，从 fiction+memory 的 git 历史
  提情绪维度）不是同一件事——**情绪画像仍归 memory 五轴档案，本命令不做情绪画像**，
  只回答「这股消耗来自哪里、落在助手哪个环节」。责任口径定死为「全部在助手一侧」，
  按 A–E 五个环节归位，不设「用户」「双方」「不明」选项。凭据沿用 language 的语义层
  （`LLM_API_KEY` → `~/.pi/agent/auth.json` → `~/.bashrc` 的 `DEEPSEEK_API_KEY`），
  `temperature=0`、只收 JSON。新增依赖 `rusqlite`（bundled）读会话库
- 判定口径可外置：`--prompt` → `QTFOUNDER_HEALTH_PROMPT` → 源码树 → `~/.config/qtfounder/health/`，
  四道都没有就报错。与 `code`／`language` 的契约同规矩，装成 CLI 之后脱离源码树也能跑
- `docs/user-guide/health.md`：命令、参数、退出码、判定口径、五环节、凭据、隐私边界、已知偏差

### Fixed

- 会话库读取：`messages.id` 与 `timestamp` 按 `cast` 取。原来按 text/real 直取会
  `Invalid column type` 直接失败——真机上跑过一次才暴露

## [cli/v0.0.1] - 2026-10-22

`src/cli` 从脚手架变成能用的工具：两个子命令查两类介质，每个介质的验收标准写成一份可在运行时替换的契约。

### Added

- CLI `code` 子命令：代码语用检查四层（词汇／句法／语篇／语用）。解析与判定分开——判定层只读 `ir::Facts`；Rust 走 `syn`、Python 走 `tree-sitter`，Go／TypeScript／Dart 共用一份通用提取器、差异全在 `specs.rs` 的一份 `LangSpec` 里，加语言只写配置。五语言一致性由测试守住。阈值在 `src/cli/src/code/default-contract.toml`——一份磁盘上的软件工程契约，运行时按 `--rules`／`QTFOUNDER_CODE_CONTRACT`／逐级往上找的顺序定位，**不内置兜底**，找不到就报错。`unused_imports` 刻意不判——rustc 已权威负责
- CLI `language` 子命令：文字审计七层（落点／自造术语／体系词／数字／句法／表格／语用），契约在 `src/cli/src/language/default-contract.toml`。带 `--sense` 语义层时只把规则挑出的段送模型，`temperature=0`、只收 JSON、直连 HTTP 不依赖内部 SDK；判不了归人。与迁移前的 Python 实现实测对等：同一份语料、两种扫描范围下命中数与按类分布完全相同
- `docs/` 三组文档：用户指南两篇（`code`／`language`，以自举方式介绍作用——拿工具查它自己的源码与文档）、开发指南按模块分文件夹（通用四篇 + `code/` + `language/`）、接口参考两篇（参数／退出码／JSON／稳定性承诺）

### Removed

- CLI `health` 子命令（check/track/history/profile）：默认路径随 `docs/` → `assets/` 迁移失效、无使用痕迹，情绪画像归 memory 五轴档案（`profile/emotions.md`）维护

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
