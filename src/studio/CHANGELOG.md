# Changelog

studio 的版本记录，仓库级版本见 [仓库 CHANGELOG](../../CHANGELOG.md)。

## [Unreleased]

### Added

- 导航栏与路由：`lib/main.dart` 改为应用壳——左侧导航栏（`数据`、`资产`），命名路由 `/data`、`/assets`，内容区 `IndexedStack` 切换，来回切不丢批次状态；原首页入口列表移除
- `lib/data/` 数据标注模块（journal 段落）：`JournalRepository` 解析仓库根（同资产抽查的解析方式），递归扫 `data/journal/**/*.md`，按空行与 `---` 切段（纯标题段丢弃），段 id 为「相对路径#段号」；批次随机抽 20 段，逐段贴「事实 / 感受 / 需要 / 行动」四类标签（取自页面规划的思考页方向）或跳过；栈空进保存页过一遍后落盘 `.annotations/journal.jsonl`（JSON Lines，追加不覆盖）
- 模块文档 `doc/data.md`：切段规则、标签集、结果格式与边界；ROADMAP、planned-ui、架构与索引文档同步当前形态

## [0.0.1] - 2026-10-28

初版：骨架入口 + 资产抽查模块。

### Added

- 首页入口列表：`lib/main.dart` 渲染功能入口，点「资产抽查」进入抽查流程
- `lib/asset/` 资产抽查模块（bloc 架构）：扫描 quanttide-founder 的文档与代码（`.md` 与常见代码后缀，跳过 `.git`、`build`、`node_modules`、`target` 等目录），批次卡片流决策保留或删除；删除的移入仓库根 `.trash/`，保留相对路径，重名加时间戳前缀，优先 `rename`、跨设备回退复制校验。仓库根默认从当前目录向上解析，可用 `--dart-define=QTFOUNDER_ROOT` 指定
- 模块文档 `doc/asset.md`：扫描范围、状态机、文件操作与边界

### Changed

- README、ROADMAP、架构与索引文档按当前结构重写；归档 models/screens 的设计合并到 `doc/data-sources.md` 与 `doc/planned-ui.md`

### Removed

- review 代码清空：`packages/review_tool/`、`lib/review/`、相关测试与 `doc/review.md` 移除，`pubspec.yaml` 去掉 `review_tool` 依赖
- 业务代码（bloc / models / repositories / screens / services / config）与测试归档至 `data/archive/platform/qtfounder/studio/`
- 过期设计文档 `doc/models/`、`doc/screens/`、`navigation.md`、`router.md`
