# API 参考

qtfounder 的接口目前只有 Provider API 一类：

## Provider API（HTTP 服务）

`src/provider/`（Go）——创作数据服务，供 Studio Web 端与外部调用方读取 fiction/memory 数据。

| 端点 | 说明 |
|------|------|
| `GET /health` | 健康检查 |
| `GET /chapters` | 改稿章节列表（fiction/职场言情/4_改稿） |
| `GET /memory` | memory 文档列表（roadmap/ + context/） |

详细定义见 [provider.md](provider.md)。

## 数据源

fiction + memory（环境变量 `QTFOUNDER_FICTION_PATH` / `QTFOUNDER_MEMORY_PATH`）。

CLI（`src/cli/`）现为脚手架、暂无子命令，新功能上线后再补参考页。
