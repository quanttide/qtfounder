import 'package:flutter_test/flutter_test.dart';
import 'package:review_tool/review_tool.dart';

Article _article(String id, {double ai = 0, String content = 'line1\nline2'}) {
  return Article(
    id: id,
    absolutePath: '/kb/$id',
    title: id,
    content: content,
    hash: hashOf(content),
    lastModified: DateTime(2026, 1, 1),
    aiProbability: ai,
  );
}

class _FakeSource implements ArticleSource {
  _FakeSource(this.articles);

  final List<Article> articles;

  @override
  Future<List<Article>> scan({required String root, Set<String>? scope}) async {
    return articles
        .where(
          (article) =>
              scope == null ||
              scope.isEmpty ||
              scope.any(
                (entry) =>
                    article.id == entry || article.id.startsWith('$entry/'),
              ),
        )
        .toList(growable: false);
  }

  @override
  Future<Article> read(String root, ArticleId id) async =>
      articles.firstWhere((article) => article.id == id);
}

class _FakeStore implements ReviewStore {
  final List<ReviewRecord> records = [];
  final Map<String, BatchMeta> batches = {};
  final Set<ArticleId> reviewed = {};

  @override
  Future<void> init() async {}

  @override
  Future<ReviewRecord?> findExisting({
    required ArticleId articleId,
    required ContentHash originalHash,
  }) async {
    for (final record in records) {
      if (record.articleId == articleId &&
          record.originalHash == originalHash) {
        return record;
      }
    }
    return null;
  }

  @override
  Future<void> save(ReviewRecord record) async {
    records.removeWhere((existing) => existing.id == record.id);
    records.add(record);
    reviewed.add(record.articleId);
  }

  @override
  Future<List<ReviewRecord>> history({int limit = 100}) async =>
      records.reversed.take(limit).toList(growable: false);

  @override
  Future<BatchMeta?> batchMeta(BatchId batchId) async => batches[batchId];

  @override
  Future<void> saveBatchMeta(BatchMeta meta) async {
    batches[meta.id] = meta;
  }

  @override
  Future<List<BatchMeta>> pendingBatches() async => batches.values
      .where((batch) => !batch.isComplete)
      .toList(growable: false);

  @override
  Future<Set<ArticleId>> reviewedArticleIds() async => reviewed;

  @override
  Future<void> close() async {}
}

ReviewService _service({List<Article>? articles, _FakeStore? store}) {
  return ReviewService(
    root: '/kb',
    source: _FakeSource(articles ?? [_article('a.md'), _article('b.md')]),
    store: store ?? _FakeStore(),
  );
}

void main() {
  group('抽样', () {
    test('Article 默认按 AI 原生处理', () {
      final article = Article(
        id: 'a.md',
        absolutePath: '/kb/a.md',
        title: 'A',
        content: '',
        hash: hashOf(''),
        lastModified: DateTime(2026, 1, 1),
      );

      expect(article.aiProbability, 1);
    });

    test('固定 seed 结果可复现', () async {
      final store = _FakeStore();
      final service = _service(store: store);

      final first = await service.createBatch(
        const SampleConfig(count: 2, seed: 7),
      );
      final second = await service.createBatch(
        const SampleConfig(count: 2, seed: 7),
      );

      expect(first.articleIds, second.articleIds);
      expect(store.batches.keys, containsAll([first.id, second.id]));
    });

    test('aiFirst 只改顺序', () async {
      final articles = [
        _article('a.md', ai: 0.9),
        _article('b.md', ai: 0.1),
        _article('c.md', ai: 0.5),
      ];
      final service = _service(articles: articles);

      final batch = await service.createBatch(
        const SampleConfig(count: 3, seed: 1, aiFirst: true),
      );

      expect(batch.articles.first.article.aiProbability, 0.9);
      expect(batch.articles.last.article.aiProbability, 0.1);
    });

    test('aiFirst 不改变入选集合', () async {
      final articles = [
        _article('a.md', ai: 0.9),
        _article('b.md', ai: 0.1),
        _article('c.md', ai: 0.5),
      ];
      final service = _service(articles: articles);

      final first = await service.createBatch(
        const SampleConfig(count: 2, seed: 3, aiFirst: true),
      );
      final second = await service.createBatch(
        const SampleConfig(count: 2, seed: 3, aiFirst: false),
      );

      expect(first.articleIds.toSet(), second.articleIds.toSet());
    });

    test('includeReviewed 控制是否排除已审文章', () async {
      final store = _FakeStore()..reviewed.add('a.md');
      final service = _service(store: store);

      final excluded = await service.createBatch(const SampleConfig(count: 10));
      expect(excluded.articleIds, isNot(contains('a.md')));

      final included = await service.createBatch(
        const SampleConfig(count: 10, includeReviewed: true),
      );
      expect(included.articleIds, contains('a.md'));
    });

    test('scope 按目录边界过滤', () async {
      final service = _service(
        articles: [_article('a.md'), _article('sub/b.md')],
      );

      final batch = await service.createBatch(
        const SampleConfig(count: 10, scope: {'sub'}),
      );
      expect(batch.articleIds, ['sub/b.md']);

      final prefixOnly = await service.createBatch(
        const SampleConfig(count: 10, scope: {'s'}),
      );
      expect(prefixOnly.articleIds, isEmpty);
    });

    test('stratum 权重随 aiProbability 提高', () async {
      final articles = [
        _article('a.md', ai: 1),
        _article('b.md', ai: 0),
        _article('c.md', ai: 0.5),
      ];
      final service = _service(articles: articles);

      final batch = await service.createBatch(
        const SampleConfig(count: 3, seed: 1, stratum: Stratum.aiProbability),
      );
      final weights = {
        for (final sampled in batch.articles)
          sampled.article.id: sampled.weight,
      };

      expect(weights['a.md']!, greaterThan(weights['c.md']!));
      expect(weights['c.md']!, greaterThan(weights['b.md']!));
    });

    test('count 必须大于 0', () async {
      final service = _service();
      expect(
        () => service.createBatch(const SampleConfig(count: 0)),
        throwsA(isA<ReviewValidationError>()),
      );
    });
  });

  group('提交', () {
    test('noIssue 计入分母', () async {
      final store = _FakeStore();
      final service = _service(store: store);
      final batch = await service.createBatch(
        const SampleConfig(count: 2, seed: 1),
      );
      final article = batch.articles.first.article;

      final result = await service.submitReview(
        ReviewRecord(
          id: 'r1',
          batchId: batch.id,
          articleId: article.id,
          originalHash: article.hash,
          revisedHash: article.hash,
          tags: const [],
          outcome: ReviewOutcome.noIssue,
          activeMs: 500,
        ),
      );

      expect(result, isA<ReviewSubmitOk>());
      expect((await service.history()).length, 1);
      expect((await store.batchMeta(batch.id))!.reviewed, 1);
    });

    test('原文被改动时返回 conflict，不落库', () async {
      final store = _FakeStore();
      final service = _service(store: store);
      final batch = await service.createBatch(
        const SampleConfig(count: 2, seed: 1),
      );
      final article = batch.articles.first.article;

      final result = await service.submitReview(
        ReviewRecord(
          id: 'r1',
          batchId: batch.id,
          articleId: article.id,
          originalHash: 'WRONG_HASH',
          revisedHash: hashOf('changed'),
          tags: const [],
          outcome: ReviewOutcome.fixed,
          activeMs: 10,
        ),
      );

      expect(result, isA<ReviewSubmitConflict>());
      expect(await service.history(), isEmpty);
    });

    test('批次不存在返回 unknown', () async {
      final service = _service();
      final article = _article('a.md');

      final result = await service.submitReview(
        ReviewRecord(
          id: 'r1',
          batchId: 'nonexistent',
          articleId: article.id,
          originalHash: article.hash,
          revisedHash: article.hash,
          tags: const [],
          outcome: ReviewOutcome.noIssue,
          activeMs: 10,
        ),
      );

      expect(result, isA<ReviewSubmitUnknownBatch>());
    });

    test('文章不在批次内时拒绝提交', () async {
      final store = _FakeStore();
      final service = _service(articles: [_article('a.md')], store: store);
      final batch = await service.createBatch(
        const SampleConfig(count: 1, seed: 1),
      );
      final outside = _article('b.md');

      final result = await service.submitReview(
        ReviewRecord(
          id: 'r1',
          batchId: batch.id,
          articleId: outside.id,
          originalHash: outside.hash,
          revisedHash: outside.hash,
          tags: const [],
          outcome: ReviewOutcome.noIssue,
          activeMs: 10,
        ),
      );

      expect(result, isA<ReviewSubmitNotInBatch>());
    });

    test('同一篇同一版本重复提交只保留一条', () async {
      final store = _FakeStore();
      final service = _service(store: store);
      final batch = await service.createBatch(
        const SampleConfig(count: 1, seed: 1),
      );
      final article = batch.articles.first.article;

      ReviewRecord record(String id) => ReviewRecord(
            id: id,
            batchId: batch.id,
            articleId: article.id,
            originalHash: article.hash,
            revisedHash: hashOf('revised'),
            tags: const [],
            outcome: ReviewOutcome.fixed,
            activeMs: 10,
          );

      final first = await service.submitReview(record('r1'));
      final second = await service.submitReview(record('r2'));

      expect(first, isA<ReviewSubmitOk>());
      expect(second, isA<ReviewSubmitOk>());
      expect((second as ReviewSubmitOk).record.id, 'r1');
      expect(store.records.length, 1);
      expect(store.records.single.id, 'r1');
    });

    test('activeMs 不能为负数', () async {
      final service = _service();
      expect(
        () => service.submitReview(
          ReviewRecord(
            id: 'r1',
            batchId: 'b1',
            articleId: 'a.md',
            originalHash: 'x',
            revisedHash: 'y',
            tags: const [],
            outcome: ReviewOutcome.noIssue,
            activeMs: -1,
          ),
        ),
        throwsA(isA<ReviewValidationError>()),
      );
    });
  });

  group('批次与序列化', () {
    test('reviewed 只统计批次内文章', () {
      final meta = BatchMeta(
        id: 'b1',
        createdAt: DateTime.fromMillisecondsSinceEpoch(1),
        articleIds: const ['a.md', 'b.md'],
        reviewedIds: const {'a.md', 'outside.md'},
      );

      expect(meta.total, 2);
      expect(meta.reviewed, 1);
      expect(meta.isComplete, isFalse);
    });

    test('BatchMeta JSON 往返', () {
      final meta = BatchMeta(
        id: 'b1',
        createdAt: DateTime.fromMillisecondsSinceEpoch(1000),
        articleIds: const ['a.md', 'b.md'],
        reviewedIds: const {'a.md'},
      );

      final restored = BatchMeta.fromJson(meta.toJson());
      expect(restored.id, meta.id);
      expect(restored.createdAt, meta.createdAt);
      expect(restored.articleIds, meta.articleIds);
      expect(restored.reviewedIds, meta.reviewedIds);
    });

    test('ReviewRecord JSON 往返保留标签', () {
      final record = ReviewRecord(
        id: 'r1',
        batchId: 'b1',
        articleId: 'a.md',
        originalHash: 'o',
        revisedHash: 'r',
        baseCommit: 'abc',
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
        activeMs: 123,
        submittedAt: DateTime.fromMillisecondsSinceEpoch(2000),
      );

      final restored = ReviewRecord.fromJson(record.toJson());
      expect(restored.tags.single.code, TagCode.aiHallucination);
      expect(restored.tags.single.startLine, 3);
      expect(restored.tags.single.anchorText, 'anchor');
      expect(restored.submittedAt, record.submittedAt);
    });
  });

  group('hash 与 diff', () {
    test('中文按 UTF-8 计算', () {
      expect(
        hashOf('中文'),
        '72726d8818f693066ceb69afa364218b692e62ea92b385782363780f47529c21',
      );
    });

    test('全量替换保持删除在前、新增在后', () async {
      final diff = await const LineDiffService().diff(
        'a1\na2\na3',
        'b1\nb2\nb3',
      );

      expect(diff, isNotNull);
      expect(diff!.indexOf('- a1'), lessThan(diff.indexOf('+ b1')));
      expect(diff, contains('- a2'));
      expect(diff, contains('+ b2'));
    });

    test('插入不会把相邻行误报为删除', () async {
      final diff = await const LineDiffService().diff('a1\na2', 'a1\nx\na2');

      expect(diff, isNotNull);
      expect(diff, contains('+ x'));
      expect(diff, isNot(contains('- a1')));
    });

    test('删除不会把相邻行误报为新增', () async {
      final diff = await const LineDiffService().diff('a1\nx\na2', 'a1\na2');

      expect(diff, isNotNull);
      expect(diff, contains('- x'));
      expect(diff, isNot(contains('+ a1')));
    });

    test('中间修改', () async {
      final diff = await const LineDiffService().diff('a\nold\nb', 'a\nnew\nb');

      expect(diff, contains('- old'));
      expect(diff, contains('+ new'));
    });

    test('空文件', () async {
      expect(await const LineDiffService().diff('', ''), isNull);
      expect(await const LineDiffService().diff('', 'x'), '+ x');
      expect(await const LineDiffService().diff('x', ''), '- x');
    });

    test('超过行数上限返回 null', () async {
      final before = List.generate(2001, (index) => 'line$index').join('\n');
      final after = List.generate(2001, (index) => 'other$index').join('\n');

      expect(await const LineDiffService().diff(before, after), isNull);
    });
  });
}
