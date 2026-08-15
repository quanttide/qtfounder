# CHANGELOG

## [0.1.0] - 2026-08-15

### Added

- src/provider：Go 创作数据 API（参考 qtdata provider 模式）
  - `GET /api/chapters`：改稿章节（fiction/职场言情/4_改稿，数值排序）
  - `GET /api/memory`：memory 文档（roadmap/ + context/）
  - `GET /health`
  - 数据源环境变量与 Studio 共用（QTFOUNDER_FICTION_PATH / QTFOUNDER_MEMORY_PATH）
  - 真实数据源测试（章节排序 + 文档读取）
