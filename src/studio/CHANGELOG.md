# Changelog

studio 的版本记录，仓库级版本见 [仓库 CHANGELOG](../../CHANGELOG.md)。

## [Unreleased]

### Added

- `lib/data/` 改为标注规则设计工具：规则列表与编辑器（规则名、标签集——每格机器名/显示名/定义/正例/反例、批次），试标 3 段真样本验规则（不落盘），保存规则与导出任务包；与 CLI `qtfounder data` 共用 XDG 存储（`QTFOUNDER_DATA` → `$XDG_DATA_HOME/qtfounder` → `~/.local/share/qtfounder`），校验与 CLI 同一把尺子
- 新增 `DataStore`（规则与任务包读写）、`RulesCubit`（列表）、`EditorCubit`（草稿/试标/保存/导出）；`DataStore.rootFrom` 纯函数钉住三级解析

### Changed

- `JournalRepository` 只读：去掉标注落盘（`save`），标注结果由标注端产出，经 `qtfounder data result import` 进存储；模块文档与架构、规划、路线图、README 同步改说法
- 原段落标注流（`DataCubit`、标签页、保存页）与 `AnnotateLabel` 四类硬编码标签移除：标签集从规则读，不再写死在代码里

## [0.0.1] - 2026-10-29

首版：骨架入口 + 导航栏 + 资产抽查与数据标注两个模块。

### Added

- 导航栏与路由：`lib/main.dart` 为应用壳——左侧导航栏（`数据`、`资产`），命名路由 `/data`、`/assets`，内容区 `IndexedStack` 切换，来回切不丢批次状态
- `lib/data/` 数据标注模块（journal 段落）：`JournalRepository` 解析仓库根，递归扫 `data/journal/**/*.md`，按空行与 `---` 切段（纯标题段丢弃），段 id 为「相对路径#段号」；批次随机抽 20 段，逐段贴「事实 / 感受 / 需要 / 行动」四类标签或跳过；栈空进保存页过一遍后落盘 `.annotations/journal.jsonl`（JSON Lines，追加不覆盖）
- `lib/asset/` 资产抽查模块（bloc 架构）：扫描 quanttide-founder 的文档与代码（`.md` 与常见代码后缀，跳过 `.git`、`build`、`node_modules`、`target` 等目录），批次卡片流决策保留或删除；删除的移入仓库根 `.trash/`，保留相对路径，重名加时间戳前缀，优先 `rename`、跨设备回退复制校验。仓库根默认从当前目录向上解析，可用 `--dart-define=QTFOUNDER_ROOT` 指定
- 模块文档 `doc/data.md` 与 `doc/asset.md`：切段与扫描规则、标签集、状态机、结果格式与边界

### Changed

- README、ROADMAP、架构与索引文档按当前结构重写；归档 models/screens 的设计合并到 `doc/data-sources.md` 与 `doc/planned-ui.md`

### Removed

- review 代码清空：`packages/review_tool/`、`lib/review/`、相关测试与 `doc/review.md` 移除，`pubspec.yaml` 去掉 `review_tool` 依赖
- 业务代码（bloc / models / repositories / screens / services / config）与测试归档至 `data/archive/platform/qtfounder/studio/`
- 过期设计文档 `doc/models/`、`doc/screens/`、`navigation.md`、`router.md`
