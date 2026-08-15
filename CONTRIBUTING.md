# CONTRIBUTING

面向 qtfounder 的**贡献者**：参与开发的规则与约定。开发环境搭建与命令见 [docs/dev-guide/index.md](docs/dev-guide/index.md)。

## 文档定位

| 文档 | 回答的问题 |
|------|-----------|
| [docs/dev-guide/index.md](docs/dev-guide/index.md) | 如何开发（结构/环境/命令） |
| 本文件 | 如何合规地贡献（规则/约定） |

## 提交约定

遵循 Conventional Commits：

| 类型 | 说明 | 示例 |
|------|------|------|
| `feat` | 新功能 | `feat: 章节详情页` |
| `fix` | 修复 | `fix: 章节排序字典序错误` |
| `docs` | 文档 | `docs: 更新 user-guide` |
| `refactor` | 重构 | `refactor: 数据加载解耦` |
| `test` | 测试 | `test: 章节加载用例` |
| `chore` | 构建/工具 | `chore: 更新依赖` |

## 提交前验证（门禁）

| 模块 | 必跑命令 |
|------|---------|
| studio | `flutter analyze && flutter test` |
| provider | `go build ./... && go vet ./... && go test ./...` |
| cli | `cargo build && cargo test` |
| site | `npm run build` |

本地双绿（格式 + lint 通过）约等于 CI 绿；CI 失败先查是格式差异还是逻辑错误。

## 分层提交

1. 模块内（src/studio 等）提交推送
2. 父仓库（quanttide-founder）`chore: update qtfounder submodule` 更新指针并推送
3. 禁止在子仓库做父仓库的事，反之亦然

## 文档同步要求

重要变更（新功能/结构变化/方向调整）后，同步更新：

- `STATUS.md`（模块版本/状态）
- `CHANGELOG.md`（版本记录）
- 对应模块 `ROADMAP.md`（任务勾选/新增）

## 数据源一致

环境变量命名（`QTFOUNDER_*`）跨模块统一，不另起名。配置方法见 dev-guide。

## 目录即语义

- 有实体才建目录；不擅自移除空目录和占位文件
- 修改前确认"改内容 vs 改名字"、"替换 vs 并存"
