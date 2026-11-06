# Changelog

## [1.0.0] - 2026-11-06

### Added

- 接入 provider：`src/api/provider.ts` 客户端（`Authorization: Bearer <JWT>`、超时与 400/401/非 200 三态错误），首页渲染改稿章节与 memory 文档两份列表，含加载、未配置、失败重试状态
- `env.local` 持有 `PROVIDER_BASE_URL` / `PROVIDER_TOKEN`（`*.local` 已在 .gitignore），构建时经 `config/index.ts` 的 defineConstants 注入；模板 `env.local.example`
- H5 端：`build:h5` / `dev:h5` 脚本与 `@tarojs/plugin-platform-h5`，静态产物独立输出 `dist-h5/`（不覆盖小程序 `dist/`）
- 数据防御：`chapters` / `docs` 为空或缺字段时回退 `[]`——上游空表曾以 `null` 返回并把整页渲染掀掉
- 发布链路（照 qtclass 惯例）：`lite/v*` tag 触发 `Deploy Lite` 的 H5 任务——构建 `dist-h5` 上传 OSS `qtfounder-lite`（分缓存）＋刷新 CDN，上线 `lite.founder.quanttide.com`；小程序任务未配密钥时改为跳过而非失败
