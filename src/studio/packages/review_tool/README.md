# review_tool

知识库抽样式审阅服务，供 `studio` 使用。

## 能力

- 按目录扫描 Markdown，`scope` 限定范围，跳过 `.git`、`node_modules`、隐藏目录与隐藏文件
- 抽样：固定 seed 可复现；`Stratum.aiProbability` 做加权无放回；`aiFirst` 只改批内顺序
- 批次：`createBatch` 生成 `BatchId` 并落 `BatchMeta`，可恢复未完成批次、算分母
- 提交：校验批次成员、原文 hash、幂等（同一 `articleId + originalHash` 只保留一条）
- 统计：`history` / `pendingBatches`；标签带行号、锚文本、备注
- diff：行级 LCS，超过 `maxLines` 返回 `null`，避免长文卡死

## 平台

| 平台 | 文件源 | 存储 | diff |
|------|--------|------|------|
| Linux / Windows / macOS / Android / iOS | `FsArticleSource` | `JsonlReviewStore` | 可用 |
| Web | unsupported | unsupported | 可用（纯 Dart） |

存储用追加式 JSONL（`records.jsonl` + `batches.jsonl`），不引入原生 SQLite 依赖。
审阅记录量级小，JSONL 可读、可 diff；需要复杂查询时再把它当源建索引。

`FsArticleSource` 只读，不写回原文。写回由调用方确认后自行处理。

## 用法

```dart
import 'package:review_tool/review_tool.dart';

final service = ReviewService(
  root: '/path/to/kb',
  source: const FsArticleSource(),
  store: JsonlReviewStore(directory: '/path/to/kb/.review'),
);
await service.init();

final batch = await service.createBatch(
  const SampleConfig(
    count: 10,
    seed: 42,
    aiFirst: true,
    stratum: Stratum.aiProbability,
  ),
);

final article = batch.articles.first.article;
final result = await service.submitReview(
  ReviewRecord(
    id: 'review-1',
    batchId: batch.id,
    articleId: article.id,
    originalHash: article.hash,
    revisedHash: hashOf(revised),
    tags: const [
      IssueTag(
        code: TagCode.aiHallucination,
        severity: IssueSeverity.major,
        startLine: 12,
      ),
    ],
    outcome: ReviewOutcome.fixed,
    activeMs: 65000,
  ),
);

if (result is ReviewSubmitConflict) {
  // 原文在审阅期间被改动，重新打开后再提交。
}
```

## AI 来源

`aiProbability` 默认 0（未知）。要让 `Stratum.aiProbability` / `aiFirst` 真正生效，需要把来源接进来：

```dart
final source = FsArticleSource(
  aiProbabilityOf: (id, content) =>
      content.contains('generated_by: ai') ? 1.0 : 0.0,
);
```

没有可信来源时，把 `aiProbability` 保持 0，不要默认归为“非 AI”。

## 验证

```bash
cd apps/qtfounder/src/studio/packages/review_tool
flutter analyze
flutter test
```
