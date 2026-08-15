# AGENTS.md — qtfounder 工作指南

## 相关文档

| 文档 | 用途 |
|------|------|
| [README](README.md) | 项目概述、四模块索引、快速开始 |
| [STATUS](STATUS.md) | 项目状态、模块版本、数据源 |
| [CONTRIBUTING](CONTRIBUTING.md) | 数据源配置方法、贡献流程 |
| [CHANGELOG](CHANGELOG.md) | 版本变更记录 |

## 项目定位

量潮创始人项目，主题为**创作现场**：创作流时间线 / 创作域矩阵 / 改稿轨迹。数据源为 fiction（小说仓库）与 memory（创始人记忆仓库），通过环境变量配置。

## 模块结构

| 模块 | 技术 | 职责 |
|------|------|------|
| `src/site/` | React + Vite | 创始人官网（创作现场展示） |
| `src/studio/` | Flutter | 创始人工作台（数据源：fiction + memory） |
| `src/provider/` | Go | 创作数据 API（chapters / memory / health） |
| `src/cli/` | Rust | CLI 工具（health 情绪分析） |

## 工作原则

1. **最小干预**：仅在用户明确请求时操作
2. **原子提交**：每次提交独立完整，验证后再提交
3. **验证优先**：修改后运行对应模块验证（flutter analyze/test、go build/vet/test、cargo build/test）
4. **分层提交**：模块内提交推送 → 父仓库（quanttide-founder）更新指针
5. **数据源一致**：环境变量命名（QTFOUNDER_*）跨模块统一，不另起名
6. **文档同步**：重要变更同步更新 STATUS.md / CHANGELOG.md

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
