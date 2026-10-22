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

`src/cli/`（Rust）——三个子命令，都是「输入一批东西、输出一份问题清单」的检查器形态：
`code`（代码四层）、`language`（文字七层）、`health`（消耗归因与责任环节）。
参数与退出码见各用户指南：[code](user-guide/code.md)、[language](user-guide/language.md)、
[health](user-guide/health.md)。退出码约定统一：0 无问题、1 有问题、2 跑不起来。
