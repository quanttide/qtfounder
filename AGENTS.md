# AGENTS.md — qtfounder 工作指南

## 相关文档

| 文档 | 用途 |
|------|------|
| [README](README.md) | 项目概述、四模块索引、快速开始 |
| [STATUS](STATUS.md) | 项目状态、模块版本、数据源 |
| [CONTRIBUTING](CONTRIBUTING.md) | 数据源配置方法、贡献流程 |
| [CHANGELOG](CHANGELOG.md) | 版本变更记录 |

## 项目定位

量潮创始人（qtfounder）仓：**创始人第二大脑的规则化**——把协作中形成的规矩落成可执行的规则：CLI 的审计与契约、提交钩子、随发 Skill。

**走向**：整体目标是把本仓内容逐渐提交到各个 `qtcloud-*` 独立仓。先例：`meta` 已并入 `qtcloud-meta`（`cli/v0.1.0` 起 `meta` 子命令淘汰）；`infra` 归宿已定 `qtcloud-infra`。其余模块迁哪、何时迁，逐个拍板，不预设。

## 模块结构

| 模块 | 技术 | 职责 |
|------|------|------|
| `src/site/` | React + Vite | 创始人官网 |
| `src/studio/` | Flutter | 创始人工作台（重建中） |
| `src/provider/` | Go | **待删除**：创作数据 API，fiction/memory 架构已废弃 |
| `src/lite/` | Taro + React + TS | 微信小程序 |
| `src/cli/` | Rust | CLI：14 个命令组，`cli/v0.1.0`，随发 11 条 Skill |

## 工作原则

1. **最小干预**：仅在用户明确请求时操作
2. **原子提交**：每次提交独立完整，验证后再提交
3. **验证优先**：修改后运行对应模块验证（flutter analyze/test、go build/vet/test、cargo build/test）
4. **分层提交**：模块内提交推送 → 父仓库（quanttide-founder）更新指针
5. **数据源一致**：环境变量命名（QTFOUNDER_*）跨模块统一，不另起名
6. **文档同步**：重要变更同步更新 STATUS.md / CHANGELOG.md
7. **迁移取向**：新能力默认不落本仓，先归领域 `qtcloud-*`，去处没定就问；迁移动手前先定口径、只移动不改写；fiction/memory 架构已废弃，涉及只做删除或标待删除，不扩建

另：改 `src/cli` 功能时，三套文档一起改——`docs/user-guide/`（怎么用）、`docs/dev-guide/`（为什么这么设计）、`docs/api-reference/`（命令面、JSON 形、退出码）。新文件加进 `docs/myst.yml` 导航，改完跑 `qtfounder language` 与 `qtfounder docs format` 审新增行。

## 验证命令

```bash
# studio
cd src/studio && flutter analyze && flutter test && flutter build web

# provider
cd src/provider && go build ./... && go vet ./... && go test ./...

# cli
cd src/cli && cargo build && cargo test

# site
cd src/site && npm run build
```

## Git 提交规范

遵循 Conventional Commits（feat / fix / docs / refactor / chore / test）。

## 输出规范

- 文档使用 MyST Markdown
- 不使用 emoji（除非用户明确请求）
- 输出简洁，适合 CLI 显示
