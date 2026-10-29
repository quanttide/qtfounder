# 架构

## 当前分层

```text
lib/main.dart                   入口：导航栏壳与命名路由
lib/data/                      标注规则设计模块
  models/rule.dart             Rule、RuleSource、RuleLabel
  models/journal_segment.dart  JournalSegment（试标与导出的单元）
  repos/data_store.dart        XDG 存储：规则与任务包的读写
  repos/journal_repository.dart  仓库根解析、journal 切段（只读）
  states/rules_cubit.dart      规则列表状态
  states/editor_cubit.dart     编辑器状态：草稿、试标、保存、导出
  screens/                     规则列表、编辑器、试标页
lib/asset/                     资产抽查模块
  models/asset_file.dart       AssetFile、AssetKind、Decision
  repos/asset_repository.dart  仓库根解析、扫描、预览、移入回收目录
  states/asset_cubit.dart      批次状态机
  screens/                     卡片流、确认页、结果页、状态分发
assets/contracts/              数据契约
```

`lib/main.dart` 只负责应用壳（导航栏 + 命名路由 `/data`、`/assets`，内容区 `IndexedStack` 切换）；业务页面按模块逐步重建。`lib/data/` 是标注规则设计工具（不执行标注，任务包导出给 lite），`lib/asset/` 是参考 recaller 思路实现的资产抽查工具，分别详见[数据：标注规则设计](data.md)与[资产抽查](asset.md)。

## 依赖方向

```text
main -> data/screens (DataAnnotatePage)、asset/screens (AssetAuditPage)
data/screens -> data/states -> data/repos -> data/models
asset/screens -> asset/states -> asset/repos -> asset/models
```

上层依赖下层，`models` 不依赖任何层。两个 `Repository` 只依赖 Dart 与 `path`，不碰平台分支；当前是桌面/Web 都可用的纯文件操作，未来接 Provider 时只替换 `repos` 层两端。

## 数据契约

`assets/contracts/` 保存 fiction 与 memory 的数据契约，当前运行时不读取；解读方式见 [数据源与契约](data-sources.md)。

## 归档业务层

`data/archive/platform/qtfounder/studio/` 保存归档的 bloc / models / repositories / screens / services / config。它们是重建参考，不是当前运行代码；当前文档不把其中的类当作现有 API。

## 重建约束

重建时在 `lib/` 下按模块分层添加（models / repos / states / screens），与 `lib/data/`、`lib/asset/` 的结构一致；先恢复数据层，再做页面，不把归档代码整块搬回。
