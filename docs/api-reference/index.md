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

## CLI（命令行）

`src/cli/`（Rust）——四个子命令。`code`（代码四层）、`language`（文字六层）、`docs`（文档结构五层）是检查器，
`think` 管认知：`load` 看消耗落在哪，`reflect` 记一条反思，`model` 看当前模型。
消耗归因的完整口径见 [think](user-guide/think.md)，其余见 CLI 自己的文档站。
退出码约定统一：0 无问题、1 有问题、2 跑不起来。
