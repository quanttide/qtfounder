# qtfounder

量潮创始人项目——**创作现场**（创作流时间线 / 创作域矩阵 / 改稿轨迹）。

## 结构

```
├── src/
│   ├── site/      → 创始人官网（React + Vite）
│   ├── studio/    → 创始人工作台（Flutter）
│   ├── provider/  → 创作数据 API（Go）
│   └── cli/       → CLI 工具（Rust）
├── STATUS.md      → 项目状态（模块版本/数据源/验证）
└── README.md
```

| 目录 | 说明 | 开发命令 |
|------|------|---------|
| `src/site/` | 创始人官网 | `cd src/site && npm run dev` |
| `src/studio/` | 创始人工作台 | `cd src/studio && flutter run` |
| `src/provider/` | 创作数据 API | `cd src/provider && go run ./cmd/server` |
| `src/cli/` | CLI 工具 | `cd src/cli && cargo run` |

## 数据源

工作台与 API 以 **fiction**（小说仓库）和 **memory**（创始人记忆仓库）为数据源，通过环境变量配置：

```bash
flutter run \
  --dart-define=QTFOUNDER_FICTION_PATH=$HOME/repos/quanttide-founder/assets/fiction \
  --dart-define=QTFOUNDER_MEMORY_PATH=$HOME/repos/quanttide-founder/assets/memory
```

配置方法详见 [CONTRIBUTING.md](CONTRIBUTING.md)（环境变量 / 数据源布局 / 平台行为）。

## 文档导航

| 文档 | 用途 |
|------|------|
| [STATUS.md](STATUS.md) | 项目状态报告（时效） |
| [CONTRIBUTING.md](CONTRIBUTING.md) | 数据源配置方法、贡献流程 |
| [AGENTS.md](AGENTS.md) | AI 工作指南 |
| [CHANGELOG.md](CHANGELOG.md) | 版本变更记录 |
| [docs/user-guide/index.md](docs/user-guide/index.md) | 用户指南（产品使用） |
| [docs/dev-guide/index.md](docs/dev-guide/index.md) | 开发指南（模块/配置/验证） |
| [docs/api-reference/index.md](docs/api-reference/index.md) | API 参考（provider 端点） |
| 各模块 ROADMAP | `src/studio/ROADMAP.md` · `src/provider/ROADMAP.md` · `src/cli/ROADMAP.md` |

## 许可

Apache 2.0
