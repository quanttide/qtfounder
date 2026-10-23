# 路线图

## 已完成

- [x] `review_tool`：核心模型、抽样、批次、提交、JSONL 存储、行级 diff
- [x] native 文件源与 Web unsupported 桩
- [x] studio path 依赖与 `createReviewService` 接线
- [x] 核心流程与原生适配器测试通过

## 下一步

- [ ] 审阅界面：文章列表、编辑器、标签与提交，等编辑器重建后接入
- [ ] AI 来源：用 frontmatter 或 manifest 提供 `aiProbabilityOf`，把默认的 AI 原生假设细化
- [ ] 数据层重建：从归档代码恢复 repository，按环境变量读取 fiction 与 memory
- [ ] 资产页：读取契约，展示 fiction 与 memory 目录树
- [ ] 导航与路由：资产、创作、思考三个职能，使用 GoRouter
- [ ] 写作页：编辑器、工作流、AI 整理层
- [ ] 思考页：日志四分类结构化

## 已知偏差

- Web 端审阅不可用，文件源与存储只在 native 实现
- 契约仍用 `memory` 命名，父仓当前数据目录是 `data/context`，重建数据层时统一
- 审阅记录默认写 `<root>/.review`，需要隔离时传 `storageDirectory`
