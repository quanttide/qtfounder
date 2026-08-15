# Studio ROADMAP

量潮创始人工作台（Flutter）——创作现场的可视化客户端。

## 目标 1：创作现场数据展示（当前）

以 fiction + memory 为数据源（环境变量 QTFOUNDER_FICTION_PATH / QTFOUNDER_MEMORY_PATH），展示创作现场。

- [x] 数据源配置（config.dart：环境变量 + 默认路径）
- [x] 数据加载（creative_repository.dart：改稿章节 + memory 文档，桌面 IO / Web 回退）
- [x] 创作现场主界面（改稿轨迹 + 创作域矩阵卡片）
- [x] 依赖注入（CreativeDesk loader 参数，测试可用）

## 目标 2：章节详情与轨迹视图

- [ ] 章节详情页：点击改稿章节 → 展示正文（对齐"改稿轨迹"定位）
- [ ] 11_x 创作轨迹：规划中章节的进度展示（roadmap/fiction.md 驱动）
- [ ] 创作域矩阵细化：fiction（职场/校园/重生）分层展示

## 目标 3：Web 端接 provider API

Web 端无文件系统——通过 src/provider 读取真实数据：

- [ ] API 客户端（http 调用 /api/chapters /api/memory）
- [ ] 数据源切换：环境变量配置 API 地址（如 QTFOUNDER_API_URL）
- [ ] Web 部署（OSS + CDN，参考 qtcloud-econ 模式）

## 验证标准

- flutter analyze 零问题
- flutter test 通过（注入式测试）
- flutter build web 成功
