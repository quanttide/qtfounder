# 架构

## 当前分层

```text
lib/main.dart                 应用壳
assets/contracts/             数据契约
```

`lib/main.dart` 只负责应用壳；业务页面尚未重建。review 代码已清空，重新设计方案确定后再按分层恢复。

## 数据契约

`assets/contracts/` 保存 fiction 与 memory 的数据契约，当前运行时不读取；解读方式见 [数据源与契约](data-sources.md)。

## 归档业务层

`data/archive/platform/qtfounder/studio/` 保存归档的 bloc / models / repositories / screens / services / config。它们是重建参考，不是当前运行代码；当前文档不把其中的类当作现有 API。

## 重建约束

重建时在 `lib/` 下按 bloc / models / repositories / screens / services 分层添加，与 `main.dart` 的注释一致；先恢复数据层，再做页面，不把归档代码整块搬回。
