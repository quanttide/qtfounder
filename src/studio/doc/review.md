# 审阅抽样

## 目的

从知识库中抽一批文章，让用户人工审阅，记录改动、标签和耗时。记录要能回答「这批抽了哪些、审了哪些、问题在哪」，而不是只存一份 diff。

## 流程

1. 扫描知识库里的 Markdown，按 `scope` 限定目录。
2. `createBatch` 按 seed 抽样，生成 `BatchId` 并保存 `BatchMeta`。
3. 用户在编辑器里修改或确认无问题。
4. `submitReview` 校验批次成员和原文 hash，再保存 `ReviewRecord`。
5. 通过 `history` 和 `pendingBatches` 查看已审与未完成批次。

## 抽样

`SampleConfig` 控制数量、seed、是否包含已审、目录范围和权重策略。固定 seed 时，扫描结果先按文章 id 排序，再抽样，保证同一输入可复现。

`Stratum.aiProbability` 做加权无放回抽样，`aiFirst` 只改批内顺序，不改变入选集合。没有 AI 来源时，`aiProbability` 默认 1，所有文章权重相同，`aiFirst` 不会产生区分。

## 提交

`ReviewRecord` 包含批次、文章、原文 hash、改后 hash、标签、结果、审阅时长和提交时间。服务会重新读取原文并比较 hash；不一致返回冲突，不落库。同一文章同一版本重复提交只保留一条记录。

标签支持问题类型、严重程度、起止行、锚文本和备注。`ReviewOutcome` 区分无问题、已修、部分修复和跳过，确保分母包含「审了但没问题」的记录。

## 存储

native 端用追加式 JSONL：`records.jsonl` 和 `batches.jsonl`。读取时按 id 去重，后写入覆盖先写入。默认目录是知识库根目录下的 `.review/`，也可以在创建服务时传 `storageDirectory`。JSONL 是可读真源，后续需要统计再加索引。

## 接入

```dart
final service = createReviewService(root: '/path/to/kb');
await service.init();

final batch = await service.createBatch(
  const SampleConfig(count: 10, seed: 42, aiFirst: true),
);
```

完整的类型、平台差异和测试命令见 [review_tool README](../packages/review_tool/README.md)。

## 边界

审阅服务不写回原文；写回由调用方确认后执行。Web 端不启用文件源和存储。diff 是展示用行级结果，不是可应用的标准 patch；超过行数上限返回 `null`。
