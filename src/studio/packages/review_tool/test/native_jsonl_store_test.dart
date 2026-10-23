import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:review_tool/review_tool.dart';

void main() {
  late Directory temp;
  late JsonlReviewStore store;

  setUp(() async {
    temp = await Directory.systemTemp.createTemp('review_store_');
    store = JsonlReviewStore(directory: temp.path);
    await store.init();
  });

  tearDown(() async {
    await store.close();
    if (await temp.exists()) await temp.delete(recursive: true);
  });

  ReviewRecord record({
    String id = 'r1',
    String articleId = 'a.md',
    String originalHash = 'orig',
    String revisedHash = 'rev',
    DateTime? submittedAt,
  }) {
    return ReviewRecord(
      id: id,
      batchId: 'b1',
      articleId: articleId,
      originalHash: originalHash,
      revisedHash: revisedHash,
      tags: const [
        IssueTag(
          code: TagCode.aiHallucination,
          severity: IssueSeverity.major,
          startLine: 3,
          endLine: 5,
          anchorText: 'anchor',
          note: 'note',
        ),
      ],
      outcome: ReviewOutcome.fixed,
      activeMs: 1234,
      submittedAt: submittedAt ?? DateTime.fromMillisecondsSinceEpoch(2000),
    );
  }

  test('批次元数据往返并参与 pending 判断', () async {
    final meta = BatchMeta(
      id: 'b1',
      createdAt: DateTime.fromMillisecondsSinceEpoch(1000),
      articleIds: const ['a.md'],
    );
    await store.saveBatchMeta(meta);

    final loaded = await store.batchMeta('b1');
    expect(loaded, isNotNull);
    expect(loaded!.articleIds, ['a.md']);
    expect(loaded.reviewedIds, isEmpty);
    expect(await store.pendingBatches(), hasLength(1));

    await store.saveBatchMeta(meta.copyWith(reviewedIds: {'a.md'}));
    expect(await store.pendingBatches(), isEmpty);
    expect((await store.batchMeta('b1'))!.reviewed, 1);
  });

  test('记录往返保留标签、时间与 hash', () async {
    await store.save(record());

    final loaded = await store.findExisting(
      articleId: 'a.md',
      originalHash: 'orig',
    );

    expect(loaded, isNotNull);
    expect(loaded!.id, 'r1');
    expect(loaded.revisedHash, 'rev');
    expect(loaded.submittedAt.millisecondsSinceEpoch, 2000);
    expect(loaded.tags.single.code, TagCode.aiHallucination);
    expect(loaded.tags.single.startLine, 3);
    expect(loaded.tags.single.anchorText, 'anchor');
    expect(await store.reviewedArticleIds(), {'a.md'});
  });

  test('history 按提交时间倒序并支持 limit', () async {
    await store.save(
      record(
        id: 'r1',
        articleId: 'a.md',
        originalHash: 'a',
        revisedHash: 'a2',
        submittedAt: DateTime.fromMillisecondsSinceEpoch(1000),
      ),
    );
    await store.save(
      record(
        id: 'r2',
        articleId: 'b.md',
        originalHash: 'b',
        revisedHash: 'b2',
        submittedAt: DateTime.fromMillisecondsSinceEpoch(2000),
      ),
    );

    final all = await store.history();
    expect(all.map((item) => item.id), ['r2', 'r1']);

    final limited = await store.history(limit: 1);
    expect(limited, hasLength(1));
    expect(limited.single.id, 'r2');
  });

  test('重复 id 以后写入为准', () async {
    await store.save(record(revisedHash: 'first'));
    await store.save(record(revisedHash: 'second'));

    final history = await store.history();
    expect(history, hasLength(1));
    expect(history.single.revisedHash, 'second');
  });

  test('关闭后重新打开仍保留记录', () async {
    await store.save(record());
    await store.close();

    final reopened = JsonlReviewStore(directory: temp.path);
    await reopened.init();

    expect(await reopened.history(), hasLength(1));
    await reopened.close();
  });
}
