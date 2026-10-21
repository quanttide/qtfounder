# Provider ROADMAP

量潮创始人工作台服务端（Go）——创作数据 API。

现状订正（2026-10-21）：目标 3 的三项均已完成而条目未勾——`internal/creative/auth.go` 实现基于 `QTFOUNDER_SECRET_KEY` 的访问限制并已在 `cmd/server/main.go` 接线（`/health` 免鉴权）；`Dockerfile` 与 `manifests/terraform` 已在仓；`.github/workflows/` 有 `ci.yml`、`deploy-provider.yml`（另有 `deploy-lite.yml`、`deploy-site.yml`）。

## 目标 1：创作数据 API（当前）

以 fiction + memory 为数据源（环境变量 QTFOUNDER_FICTION_PATH / QTFOUNDER_MEMORY_PATH，与 Studio 共用）。

- [x] `GET /chapters`：改稿章节列表（数值排序）
- [x] `GET /memory`：memory 文档列表（roadmap/ + context/）
- [x] `GET /health`：健康检查
- [x] 真实数据源测试（repository_test.go）

## 目标 2：内容接口

- [ ] `GET /chapters/{id}`：章节详情（正文内容，支持 Web 端阅读）。达成标志：返回 200 与章节正文；id 不存在返回 404，不 500
- [ ] `GET /memory/{category}/{name}`：文档内容。达成标志：返回 200 与 Markdown 原文；解析出的路径必须落在 `QTFOUNDER_MEMORY_PATH` 之内，越界路径（`..`、绝对路径）返回 400
- [ ] 章节状态标注（成稿 / 脚本 / 规划——对齐改稿轨迹）。达成标志：列表与详情都带状态字段，取值来自改稿轨迹而非人工标注

## 目标 3：生产化

- [x] 鉴权（工作台为私有数据源）：`SecretKeyAuth` 中间件，`Authorization: Bearer` 或 `?key=`，`QTFOUNDER_SECRET_KEY` 未设时不启用（本地开发）
- [x] 部署：`Dockerfile` + `manifests/terraform`
- [x] CI：`.github/workflows/ci.yml`（build/vet/test）+ `deploy-provider.yml`

## 验证标准

- go build / go vet / go test 全通过（真实数据源）
- API 实测返回真实章节与文档
