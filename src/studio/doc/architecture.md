# 架构

## 当前分层

```text
lib/main.dart                   入口与首页
lib/asset/                     资产抽查模块
  model/asset_file.dart        AssetFile、AssetKind、Decision
  data/asset_repository.dart   仓库根解析、扫描、预览、移入回收目录
  bloc/asset_cubit.dart        批次状态机
  ui/                          卡片流、确认页、结果页、状态分发
assets/contracts/              数据契约
```

`lib/main.dart` 只负责入口与首页列表；业务页面按模块逐步重建。`lib/asset/` 是参考 recaller 思路实现的资产抽查工具，详见[资产抽查](asset.md)。

## 依赖方向

```text
main -> asset/ui (AssetAuditPage)
asset/ui -> asset/bloc -> asset/data -> asset/model
```

上层依赖下层，`model` 不依赖任何层。`AssetRepository` 只依赖 Dart 与 `path`，不碰平台分支；当前是桌面/Web 都可用的纯文件操作，未来接 Provider 时只替换 `data` 层两端。

## 数据契约

`assets/contracts/` 保存 fiction 与 memory 的数据契约，当前运行时不读取；解读方式见 [数据源与契约](data-sources.md)。

## 归档业务层

`data/archive/platform/qtfounder/studio/` 保存归档的 bloc / models / repositories / screens / services / config。它们是重建参考，不是当前运行代码；当前文档不把其中的类当作现有 API。

## 重建约束

重建时在 `lib/` 下按模块分层添加（model / data / bloc / ui），与 `lib/asset/` 的结构一致；先恢复数据层，再做页面，不把归档代码整块搬回。
