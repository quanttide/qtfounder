# Changelog

## [Unreleased]

### Added

- 接入 provider：`src/api/provider.ts` 客户端（`Authorization: Bearer <JWT>`、超时与 400/401/非 200 三态错误），首页渲染改稿章节与 memory 文档两份列表，含加载、未配置、失败重试状态
- `env.local` 持有 `PROVIDER_BASE_URL` / `PROVIDER_TOKEN`（`*.local` 已在 .gitignore），构建时经 `config/index.ts` 的 defineConstants 注入；模板 `env.local.example`
