# 量潮创始人工作台

`qtfounder_studio` 是创始人工作台的 Flutter 客户端，主题是创作现场：让创作过程可见。当前处于骨架阶段：运行入口是 `lib/main.dart`，业务代码已归档，review 代码已清空，等待重新设计。

## 当前状态

`lib/main.dart` 只渲染应用壳和标题，没有导航、路由或编辑界面。归档的业务代码（bloc / models / repositories / screens / services / config）在 `data/archive/platform/qtfounder/studio/`，重建时按模块逐步恢复，不直接把归档代码搬回。

原审阅抽样包 `packages/review_tool` 及其接线已移除；`src/studio` 只保留入口，等重新设计方案确定后再按分层重建。

## 目录

```text
src/studio/
├── lib/main.dart
├── assets/contracts/
├── test/
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
flutter build web
```

## 文档

- [Studio 文档](doc/index.md)
- [架构](doc/architecture.md)
- [数据源与契约](doc/data-sources.md)
- [页面规划](doc/planned-ui.md)
- [路线图](ROADMAP.md)
