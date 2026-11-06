# Provider API 参考

创作数据 API（`src/provider/`，Go 服务）。

## 服务启动

```bash
cd src/provider
QTFOUNDER_FICTION_PATH=$HOME/repos/quanttide-founder/assets/fiction \
QTFOUNDER_MEMORY_PATH=$HOME/repos/quanttide-founder/data/context \
QTFOUNDER_ADDR=:8080 \
  go run ./cmd/server
```

## 鉴权

RS256 JWT，双层校验：FC 触发器（`manifests/terraform/jwks.json`）与应用层 `JWTAuth` 中间件，客户端只带一个令牌：

```bash
TOKEN=$(cd src/provider && go run ./cmd/mint -ttl 24h)   # 私钥默认 ~/.local/share/qtfounder/jwt/RS256_private.pem
curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:8080/chapters
```

- 只认 `Authorization` 头，`?key=` 查询参数不再放行（URL 会把密钥带进日志与历史）；
- 令牌必须带 `exp`，签名算法钉死 RS256，过期或换算法一律 401；
- `GET /healthz` 免验（FC 健康检查与探活）；
- 本地未设置 `QTFOUNDER_JWT_PUBLIC_KEY` 时不启用鉴权，仅限本地开发。

## 端点

### GET /healthz

健康检查，免鉴权。

**响应**：
```json
{"status": "ok"}
```

### GET /chapters

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

### GET /memory

memory 文档列表（journal/ + handbook/，按分类与名称排序）。数据源目录缺失或无文档时返回空数组。

**响应**：
```json
{
  "docs": [
    {"name": "2026-11-06", "category": "journal", "path": "/abs/path/journal/2026-11-06.md"},
    {"name": "api-reference", "category": "handbook", "path": "/abs/path/handbook/api-reference.md"}
  ]
}
```

| 字段 | 说明 |
|------|------|
| `name` | 文档名（去 .md） |
| `category` | 分类：`journal`（日志）/ `handbook`（手册） |
| `path` | 文件绝对路径 |

## 错误

| 状态码 | 场景 |
|--------|------|
| 401 | 缺少、过期或校验不过的 JWT（`WWW-Authenticate: Bearer`） |
| 500 | 数据源不可用（环境变量未配置 / 路径不存在） |

## 用途

- **Studio Web 端**：Web 无文件系统，经此 API 读取真实数据（见 studio ROADMAP 目标 3）
- **调试与集成**：任何需要创作数据的调用方
