# CHANGELOG

## [Unreleased]

### Changed

- 鉴权从共享静态密钥改为 RS256 JWT，双层校验（FC 触发器 + 应用层中间件）：
  - 应用层 `JWTAuth`（`internal/creative/auth.go`）只认 `Authorization: Bearer` 头，算法钉死 RS256，`exp` 必填
  - FC 触发器 `authType=jwt`，JWKS 见 `manifests/terraform/jwks.json`，`/healthz` 白名单免验
  - 新增 `cmd/mint`：用本机私钥签发令牌，公钥随 terraform 注入，私钥不进仓库
  - `QTFOUNDER_SECRET_KEY` 变量、GitHub secret 依赖与 `tfstate` 明文落盘一并移除
- `GET /health` 更名为 `GET /healthz`，FC 健康检查同步改为探 `/healthz`
- 部署：代码包 OSS 对象名带 git sha——同名对象重复上传时 FC 不会重新拉取代码，此前部署新代码可能仍跑旧版本

### Removed

- `?key=` 查询参数携带密钥的通路（URL 会把密钥带进访问日志与历史）

### Fixed

- CORS 预检 OPTIONS 在鉴权前短路返回 204（`Preflight` 中间件）：浏览器规范要求预检 2xx，此前预检落到应用层 401，跨端页面（lite H5）会被判网络错误；`Access-Control-*` 响应头由 FC 网关补

## [0.1.0] - 2026-08-15

### Added

- src/provider：Go 创作数据 API（参考 qtdata provider 模式）
  - `GET /chapters`：改稿章节（fiction/职场言情/4_改稿，数值排序）
  - `GET /memory`：memory 文档（roadmap/ + context/）
  - `GET /health`
  - 数据源环境变量与 Studio 共用（QTFOUNDER_FICTION_PATH / QTFOUNDER_MEMORY_PATH）
  - 真实数据源测试（章节排序 + 文档读取）
