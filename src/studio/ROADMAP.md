# 路线图

## 当前状态

- [x] 骨架与入口 `lib/main.dart`（业务代码已归档，见 CHANGELOG）
- [x] `lib/asset/` 资产抽查：扫描、批次卡片流、保留/删除、移入 `.trash/`、结果清单
- [x] 导航栏与路由：`数据`、`资产` 两个入口，命名路由 `/data`、`/assets`，`IndexedStack` 保状态
- [x] `lib/data/` 数据标注：journal 切段、批次流贴四类标签、落盘 `.annotations/journal.jsonl`

## 下一步

- [ ] Studio 重新设计：资产、创作、思考三个职能
- [ ] 数据层重建：从归档代码恢复 repository，按环境变量读取 fiction 与 memory
- [ ] 资产页：读取契约，展示 fiction 与 memory 目录树
- [ ] 写作页：编辑器、工作流、AI 整理层
- [ ] 标注结果消费：读 `.annotations/journal.jsonl` 做整理视图

## 已知偏差

- 契约仍用 `memory` 命名，父仓当前数据目录是 `data/context`，重建数据层时统一
- 资产抽查的删除是移入 `.trash/`，不进版本控制；真正清理需手动删 `.trash/`
