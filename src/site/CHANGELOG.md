# Changelog

## [0.1.0-alpha.3] - 2026-08-14

### Changed

- 部署形态改为域名根路径：vite `base` 改为 `/`，去掉 `BrowserRouter` basename（`/qtfounder` 前缀移除，GitHub Pages 部署已停用）
- OSS 桶布局改为桶根：`assets/`、`index.html`、`404.html` 直接位于桶根；SPA 路由（works/brain/详情页）以无扩展名 key + `Content-Type:text/html` 上传，深链直接返回 200
- 移除 GitHub Pages 部署工作流（`.github/workflows/deploy.yml`），OSS 部署（deploy-site.yml）为唯一部署链路

### Fixed

- 修复根路径访问白屏：`BrowserRouter` 固定 basename 与域名根路径 URL 不匹配导致 React Router 拒绝渲染

## [0.1.0-alpha.2] - 2026-08-14

### Fixed

- OSS 部署工作流移至 `.github/workflows/deploy-site.yml`（对齐 qtcloud `deploy-studio.yml` 模式）：此前置于仓库根目录，GitHub Actions 不识别，发布后从未自动部署
- 上传路径修正：构建产物上传到 OSS 桶 `qtfounder/` 前缀（与 vite `base=/qtfounder/` 匹配）并同步桶根 index.html；此前路径错位导致 HTML 引用的资源 404
- 缓存策略区分：哈希资源长缓存、入口文件 no-cache；新增 404.html SPA 回退

## [0.1.0-alpha.1] - 2026-08-14

结构类变更：首页从"作品集"重构为"创作现场"；作品数据模型从单类型扩展为四类型（改稿/文章/游戏/工具）；改稿正文目录迁移为按版本组织。

### Added

- 首页创作流时间线：混合类型按时间倒序，最多 10 条，附"更多 →"链接到 `/works`
- 首页创作域矩阵：小说 / 游戏 / 工具 / 思考，各含当前状态、最近动态、下一步
- 首页 Hero 实时锚点：当日创作动态摘要（静态版手填）
- 首页联系区块：GitHub 真实链接；邮件、社交为占位，待作者补充
- 作品筛选扩展：`全部 / 改稿 / 文章 / 游戏 / 工具`，选中加粗
- 详情页改稿轨迹：版本列表（版本 + 日期 + 说明）+ 版本切换
- 数据层：`works.ts` 类型扩展与 `revisions` 字段；新增 `streams.ts`、`activities.ts`

### Changed

- 改稿正文目录迁移：`content/works/fiction/<slug>.md` → `content/works/fiction/<slug>/v2.md`（最新版 v2 语言润色；v0 初稿、v1 叙事调整正文暂未收录，仅保留版本记录，待数据自动化从 git 历史生成）
- 作品列表按日期倒序排列（含跨类型混合）
- 首页"查看全部"改为"更多"，链接至 `/works`

### Removed

- 首页"最近 3 篇作品"区块（由创作流时间线替代）

## [0.0.1] - 2025-07-13

### Added

- 首页：Hero、关于、最近作品展示
- 作品列表页 `/works`：全部/改稿筛选
- 作品详情页 `/works/fiction/drafts/:slug`：展示改稿原文
- 路由系统（react-router-dom）
- 14 篇职场言情改稿作品上线
