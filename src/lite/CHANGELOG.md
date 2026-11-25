# Changelog

## [0.1.0-alpha.5] - 2026-11-25

### Fixed

- 首屏报错（旧代码打服务端）根因：Taro 产物文件名不带内容 hash，却按 `max-age=31536000` 上传——用户浏览器把旧 JS 钉一年，新 index 配旧 app.js，旧代码还在调 provider。发布改为全部 `no-cache`
- 任意路径直开吐 OSS `NoSuchKey` XML：桶补静态托管（`index.html` 兜底），深链回落首页
- `<title>` 从占位 `lite` 改为「量潮创始人 lite」

## [0.1.0-alpha.4] - 2026-11-25

### Added

- 待办每条带完整说明：`data/todos.ts` 的条目加 `detail`（多行原样换行显示），首页详情行小字灰色置于标题下

### Changed

- 改纯静态、撤掉服务端接入：首页不再请求 provider（`src/api/provider.ts` 与章节详情页 `pages/chapter` 删除，`config/index.ts` 不再注入 `PROVIDER_*`，`env.local.example` 删除），待办内容落在 `src/data/todos.ts`——改这份文件推 tag 即发布；`deploy-lite.yml` 去掉写 env.local 的步骤与 `QTFOUNDER_PROVIDER_*` 依赖
- 首页只留待办：改稿章节与 memory 文档两区移除（数据源本就读不到，页面不再受其 500 影响）

## [0.1.0-alpha.3] - 2026-11-25

### Added

- 首页新增「待办」区并置于最前：`fetchTodos()` 走 provider `GET /todos`，条目带分组标签（`## 小节`），已完成项灰色删除线，空表给「暂无待办」；与另两份列表共用加载 / 未配置 / 失败重试三态，一次 `Promise.all` 拉三份数据

## [0.1.0-alpha.2] - 2026-11-06

### Added

- 章节详情页 `pages/chapter`：点章节行进入、展示正文（markdown 原样、不引渲染库），含加载/失败重试/缺 id 三态；`fetchChapter` 走 `GET /chapters/{id}`，404 给「章节不存在或已改名」
- 修路由参数双重编码：Taro h5/weapp 给未解码原样，客户端先解码还原再统一编码，否则详情页拿到 `22_%E4%B9%A6...` 恒 404

## [0.1.0-alpha.1] - 2026-11-06

### Added

- 接入 provider：`src/api/provider.ts` 客户端（`Authorization: Bearer <JWT>`、超时与 400/401/非 200 三态错误），首页渲染改稿章节与 memory 文档两份列表，含加载、未配置、失败重试状态
- `env.local` 持有 `PROVIDER_BASE_URL` / `PROVIDER_TOKEN`（`*.local` 已在 .gitignore），构建时经 `config/index.ts` 的 defineConstants 注入；模板 `env.local.example`
- H5 端：`build:h5` / `dev:h5` 脚本与 `@tarojs/plugin-platform-h5`，静态产物独立输出 `dist-h5/`（不覆盖小程序 `dist/`）
- 数据防御：`chapters` / `docs` 为空或缺字段时回退 `[]`——上游空表曾以 `null` 返回并把整页渲染掀掉
- 发布链路（照 qtclass 惯例）：`lite/v*` tag 触发 `Deploy Lite` 的 H5 任务——构建 `dist-h5` 上传 OSS `qtfounder-lite`（分缓存）＋刷新 CDN，上线 `lite.founder.quanttide.com`；小程序任务未配密钥时改为跳过而非失败
