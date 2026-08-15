# API 参考

面向调用方：qtfounder 创作数据 API（src/provider，Go 服务）的接口定义。

## 服务启动

```bash
cd src/provider
QTFOUNDER_FICTION_PATH=$HOME/repos/quanttide-founder/assets/fiction \
QTFOUNDER_MEMORY_PATH=$HOME/repos/quanttide-founder/assets/memory \
QTFOUNDER_ADDR=:8080 \
  go run ./cmd/server
```

## 端点

### GET /health

健康检查。

**响应**：
```json
{"status": "ok"}
```

### GET /api/chapters

改稿章节列表（fiction/职场言情/4_改稿，按章节编号数值排序）。

**响应**：
```json
{
  "chapters": [
    {"id": "1_1", "title": "咖啡厅重逢", "path": "/abs/path/1_1_咖啡厅重逢.md"},
    {"id": "1_2", "title": "深夜失眠", "path": "/abs/path/1_2_深夜失眠.md"}
  ]
}
```

| 字段 | 说明 |
|------|------|
| `id` | 章节编号（如 `1_1`） |
| `title` | 章节标题（如 `咖啡厅重逢`） |
| `path` | 文件绝对路径 |

### GET /api/memory

memory 文档列表（roadmap/ + context/，按分类与名称排序）。

**响应**：
```json
{
  "docs": [
    {"name": "fiction", "category": "roadmap", "path": "/abs/path/fiction.md"},
    {"name": "fiction-plot", "category": "context", "path": "/abs/path/fiction-plot.md"}
  ]
}
```

| 字段 | 说明 |
|------|------|
| `name` | 文档名（去 .md） |
| `category` | 分类：`roadmap`（方向）/ `context`（方法） |
| `path` | 文件绝对路径 |

## 错误

| 状态码 | 场景 |
|--------|------|
| 500 | 数据源不可用（环境变量未配置 / 路径不存在） |

## 用途

- **Studio Web 端**：Web 无文件系统，经此 API 读取真实数据（见 studio ROADMAP 目标 3）
- **调试与集成**：任何需要创作数据的调用方
