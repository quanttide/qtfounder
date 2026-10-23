# 架构

## 当前分层

```text
lib/main.dart                 应用壳
lib/review/                   客户端接线
packages/review_tool/         审阅抽样
  lib/src/core/               模型、接口、服务、diff、hash
  lib/src/native/             dart:io 文件源、JSONL 存储
  lib/src/web/                unsupported 桩
assets/contracts/             数据契约
```

`lib/main.dart` 只负责应用壳；业务页面尚未重建。`lib/review/review_service_factory.dart` 把文件源和 JSONL 存储组装成 `ReviewService`。

## 依赖方向

`review_tool` 的 core 只依赖 Dart 与 `crypto`，不读文件、不碰平台。native 层实现 `ArticleSource` 和 `ReviewStore`，web 层给出同名 unsupported 桩。应用只依赖 `review_tool` 的公开入口，不直接依赖 native 实现。

```text
app -> review_tool (public)
review_tool core -> interfaces
native/web adapters -> core
```

## 审阅数据流

```text
FsArticleSource.scan
  -> ReviewService.createBatch
  -> SampleBatch + BatchMeta
  -> 用户编辑
  -> ReviewRecord
  -> ReviewService.submitReview
  -> ReviewStore (JSONL)
```

`createBatch` 会写入批次元数据，`submitReview` 校验批次成员和原文 hash，并把已审文章计入批次分母。原文只读，写回不在服务内完成。

## 平台

| 平台 | 文件源 | 存储 | 审阅 |
| --- | --- | --- | --- |
| Linux / Windows / macOS / Android / iOS | `FsArticleSource` | `JsonlReviewStore` | 可用 |
| Web | unsupported | unsupported | 不可用 |

`LineDiffService` 与 `hashOf` 是纯 Dart，Web 也可用；`flutter build web` 通过。Web 端如果展示审阅入口，需要先判断平台。

## 归档业务层

`data/archive/platform/qtfounder/studio/` 保存归档的 bloc / models / repositories / screens / services / config。它们是重建参考，不是当前运行代码；当前文档不把其中的类当作现有 API。

## 设计约束

审阅记录默认按 AI 原生假设处理，`aiProbability` 默认 1；只有可信来源能把它降到 0。存储用追加式 JSONL，不引入原生 SQLite 依赖；需要复杂查询时再把 JSONL 当源建索引。审阅功能不在 Web 端启用；原文写回由调用方确认后自行处理。
