# 量潮创始人工作台

`qtfounder_studio` 是创始人工作台的 Flutter 客户端，主题是创作现场：让创作过程可见。当前处于重建阶段：入口是 `lib/main.dart`，已落地第一个模块 `lib/asset/`（资产抽查）。

## 当前状态

`lib/main.dart` 渲染首页入口列表，点「资产抽查」进入抽查流程。首页没有导航栏、路由或编辑界面。

归档的业务代码（bloc / models / repositories / screens / services / config）在 `data/archive/platform/qtfounder/studio/`，重建时按模块逐步恢复，不直接把归档代码搬回。

`lib/asset/` 是参考 recaller 思路实现的资产抽查工具：从仓库的文档与代码里随机抽一批，卡片流决策保留或删除，删除的移入根目录 `.trash/`。详见[资产抽查](doc/asset.md)。

## 目录

```text
src/studio/
├── lib/
│   ├── main.dart                 入口与首页
│   └── asset/                    资产抽查模块
│       ├── model/asset_file.dart
│       ├── data/asset_repository.dart
│       ├── bloc/asset_cubit.dart
│       └── ui/
├── assets/contracts/
├── test/asset/
├── doc/
├── ROADMAP.md
└── pubspec.yaml
```

`assets/contracts/` 保存 fiction 与 memory 的数据契约，供重建资产页时使用；当前运行时不读取契约。

## 开发

```bash
cd apps/qtfounder/src/studio
flutter pub get
flutter analyze
flutter test
flutter run -d linux
```

仓库根默认按当前目录向上解析；也可以直接指定：

```bash
flutter run -d linux --dart-define=QTFOUNDER_ROOT=/abs/path/to/quanttide-founder
```

## 文档

- [Studio 文档](doc/index.md)
- [架构](doc/architecture.md)
- [资产抽查](doc/asset.md)
- [数据源与契约](doc/data-sources.md)
- [页面规划](doc/planned-ui.md)
- [路线图](ROADMAP.md)
