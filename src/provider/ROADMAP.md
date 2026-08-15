# Provider ROADMAP

量潮创始人工作台服务端（Go）——创作数据 API。

## 目标 1：创作数据 API（当前）

以 fiction + memory 为数据源（环境变量 QTFOUNDER_FICTION_PATH / QTFOUNDER_MEMORY_PATH，与 Studio 共用）。

- [x] `GET /api/chapters`：改稿章节列表（数值排序）
- [x] `GET /api/memory`：memory 文档列表（roadmap/ + context/）
- [x] `GET /health`：健康检查
- [x] 真实数据源测试（repository_test.go）

## 目标 2：内容接口

- [ ] `GET /api/chapters/{id}`：章节详情（正文内容，支持 Web 端阅读）
- [ ] `GET /api/memory/{category}/{name}`：文档内容
- [ ] 章节状态标注（成稿 / 脚本 / 规划——对齐改稿轨迹）

## 目标 3：生产化

- [ ] 鉴权（工作台为私有数据源，需 token 或本地网络限制）
- [ ] 部署（Dockerfile + terraform，参考 qtcloud-delib provider 模式）
- [ ] CI（ci.yml：build/vet/test + 部署 workflow）

## 验证标准

- go build / go vet / go test 全通过（真实数据源）
- API 实测返回真实章节与文档
