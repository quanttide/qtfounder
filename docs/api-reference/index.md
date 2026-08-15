# API 参考

qtfounder 的 API 总体构成——两类接口：

## 1. Provider API（HTTP 服务）

`src/provider/`（Go）——创作数据服务，供 Studio Web 端与外部调用方读取 fiction/memory 数据。

| 端点 | 说明 |
|------|------|
| `GET /health` | 健康检查 |
| `GET /api/chapters` | 改稿章节列表（fiction/职场言情/4_改稿） |
| `GET /api/memory` | memory 文档列表（roadmap/ + context/） |

详细定义见 [provider.md](provider.md)。

## 2. CLI 命令（qtfounder-cli）

`src/cli/`（Rust）——本地命令行工具，基于 git commit 历史与 LLM 做情绪维度分析。

| 命令 | 说明 |
|------|------|
| `qtfounder-cli health check` | 情绪状态检查（近 N 天） |
| `qtfounder-cli health track` | 情绪追踪（CSV 输出） |
| `qtfounder-cli health history` | 历史情绪查询 |
| `qtfounder-cli health profile` | 情绪画像生成 |

详细定义见 [cli.md](cli.md)。

## 共同点

- 数据源均为 fiction + memory（环境变量 QTFOUNDER_FICTION_PATH / QTFOUNDER_MEMORY_PATH）
- 服务型（provider）面向外部调用，工具型（cli）面向本地使用
