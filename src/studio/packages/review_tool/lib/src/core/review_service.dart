import 'dart:math';

import 'article.dart';
import 'batch.dart';
import 'diff.dart';
import 'errors.dart';
import 'interfaces.dart';
import 'record.dart';
import 'sample_config.dart';
import 'types.dart';

/// 抽样 + 提交的主流程。
///
/// 平台层只负责提供 [ArticleSource] 与 [ReviewStore]；抽样、批次、提交校验
/// 和 diff 都在这里，native 与 web 共用。
class ReviewService {
  ReviewService({
    required this.root,
    required this.source,
    required this.store,
    DiffService diffService = const LineDiffService(),
  }) : _diffService = diffService {
    if (root.trim().isEmpty) {
      throw const ReviewValidationError('知识库根目录不能为空');
    }
  }

  final String root;
  final ArticleSource source;
  final ReviewStore store;
  final DiffService _diffService;

  Future<void> init() => store.init();

  /// 抽一批文章，并立即落 [BatchMeta]，调用方拿到的 id 可直接用于提交。
  Future<SampleBatch> createBatch(SampleConfig config) async {
    if (config.count <= 0) {
      throw const ReviewValidationError('count 必须大于 0');
    }

    final all = await source.scan(root: root, scope: config.scope);
    final reviewed = config.includeReviewed
        ? const <ArticleId>{}
        : await store.reviewedArticleIds();

    final pool = all
        .where((article) => !reviewed.contains(article.id))
        .toList(growable: false)
      ..sort((a, b) => a.id.compareTo(b.id));

    final random = config.seed == null ? Random() : Random(config.seed);
    final picked = _sample(pool, config, random);

    if (config.aiFirst) {
      picked.sort((a, b) {
        final byProbability = b.article.aiProbability.compareTo(
          a.article.aiProbability,
        );
        return byProbability != 0
            ? byProbability
            : a.article.id.compareTo(b.article.id);
      });
    }

    final batch = SampleBatch(
      id: _newBatchId(),
      seed: config.seed,
      articles: picked,
    );

    await store.saveBatchMeta(
      BatchMeta(
        id: batch.id,
        createdAt: DateTime.now(),
        articleIds: batch.articleIds,
      ),
    );

    return batch;
  }

  /// 提交一次审阅。原文 hash 以磁盘当前内容为准，不信任调用方传入的 original。
  Future<ReviewSubmitResult> submitReview(ReviewRecord record) async {
    if (record.activeMs < 0) {
      throw const ReviewValidationError('activeMs 不能为负数');
    }

    final meta = await store.batchMeta(record.batchId);
    if (meta == null) return const ReviewSubmitUnknownBatch();
    if (!meta.articleIds.contains(record.articleId)) {
      return ReviewSubmitNotInBatch(record.articleId);
    }

    final original = await source.read(root, record.articleId);
    if (original.hash != record.originalHash) {
      return ReviewSubmitConflict(
        '原文已变更（hash 不匹配），请重新打开后再提交: ${record.articleId}',
      );
    }

    final existing = await store.findExisting(
      articleId: record.articleId,
      originalHash: record.originalHash,
    );
    final toSave = existing == null ? record : record.copyWith(id: existing.id);

    await store.save(toSave);
    await store.saveBatchMeta(
      meta.copyWith(reviewedIds: {...meta.reviewedIds, record.articleId}),
    );

    return ReviewSubmitOk(toSave);
  }

  Future<List<ReviewRecord>> history({int limit = 100}) =>
      store.history(limit: limit);

  Future<List<BatchMeta>> pendingBatches() => store.pendingBatches();

  Future<String?> diff(String before, String after) =>
      _diffService.diff(before, after);

  Future<void> close() => store.close();

  List<SampledArticle> _sample(
    List<Article> pool,
    SampleConfig config,
    Random random,
  ) {
    final target = min(config.count, pool.length);
    final remaining = [...pool];
    final picked = <SampledArticle>[];

    while (picked.length < target && remaining.isNotEmpty) {
      final weights = remaining
          .map((article) => _weight(article, config.stratum))
          .toList(growable: false);
      final total = weights.fold<double>(0, (sum, weight) => sum + weight);
      var cursor = random.nextDouble() * total;
      var index = 0;
      for (; index < remaining.length; index++) {
        cursor -= weights[index];
        if (cursor <= 0) break;
      }
      if (index >= remaining.length) index = remaining.length - 1;
      picked.add(SampledArticle(remaining.removeAt(index), weights[index]));
    }

    return picked;
  }

  double _weight(Article article, Stratum stratum) {
    if (stratum == Stratum.none) return 1;
    final probability = article.aiProbability.clamp(0.0, 1.0);
    return 0.1 + probability * 0.9;
  }

  String _newBatchId() {
    final timestamp = DateTime.now().microsecondsSinceEpoch.toRadixString(36);
    final salt = Random().nextInt(1 << 20).toRadixString(36);
    return 'b_${timestamp}_$salt';
  }
}
