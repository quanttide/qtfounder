# 路线图

## 当前状态

- [x] 骨架与入口 `lib/main.dart`（业务代码已归档，见 CHANGELOG）
- [x] `lib/asset/` 资产抽查：扫描、批次卡片流、保留/删除、移入 `.trash/`、结果清单
- [x] 导航栏与路由：`数据`、`资产` 两个入口，命名路由 `/data`、`/assets`，`IndexedStack` 保状态
- [x] `lib/data/` 标注规则设计：规则列表与编辑器（标签集含定义/正反例）、试标 3 段验规则、任务包导出；与 CLI `qtfounder data` 共用 XDG 存储

## 下一步

- [ ] Studio 重新设计：资产、创作、思考三个职能
- [ ] 数据层重建：从归档代码恢复 repository，按环境变量读取 fiction 与 memory
- [ ] 资产页：读取契约，展示 fiction 与 memory 目录树
- [ ] 写作页：编辑器、工作流、AI 整理层
- [ ] lite 接入：读任务包做标注，导出 JSONL 回传（`qtfounder data result import` 收）
- [ ] 标注结果消费：统计视图与规则迭代反馈

## 已知偏差

- 契约仍用 `memory` 命名，父仓当前数据目录是 `data/context`，重建数据层时统一
- 资产抽查的删除是移入 `.trash/`，不进版本控制；真正清理需手动删 `.trash/`
