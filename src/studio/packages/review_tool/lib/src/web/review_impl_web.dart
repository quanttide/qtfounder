import '../core/article.dart';
import '../core/batch.dart';
import '../core/errors.dart';
import '../core/interfaces.dart';
import '../core/record.dart';
import '../core/types.dart';

/// Web 端不读文件系统、不落本地记录。保持同名接口，业务层不用写平台判断。
class FsArticleSource implements ArticleSource {
  const FsArticleSource({this.aiProbabilityOf});

  final AiProbabilityResolver? aiProbabilityOf;

  @override
  Future<List<Article>> scan({
    required String root,
    Set<String>? scope,
  }) =>
      throw const ReviewUnsupportedError();

  @override
  Future<Article> read(String root, ArticleId id) =>
      throw const ReviewUnsupportedError();
}

class JsonlReviewStore implements ReviewStore {
  const JsonlReviewStore({this.directory});

  final String? directory;

  @override
  Future<void> init() => throw const ReviewUnsupportedError();

  @override
  Future<ReviewRecord?> findExisting({
    required ArticleId articleId,
    required ContentHash originalHash,
  }) =>
      throw const ReviewUnsupportedError();

  @override
  Future<void> save(ReviewRecord record) =>
      throw const ReviewUnsupportedError();

  @override
  Future<List<ReviewRecord>> history({int limit = 100}) =>
      throw const ReviewUnsupportedError();

  @override
  Future<BatchMeta?> batchMeta(BatchId batchId) =>
      throw const ReviewUnsupportedError();

  @override
  Future<void> saveBatchMeta(BatchMeta meta) =>
      throw const ReviewUnsupportedError();

  @override
  Future<List<BatchMeta>> pendingBatches() =>
      throw const ReviewUnsupportedError();

  @override
  Future<Set<ArticleId>> reviewedArticleIds() =>
      throw const ReviewUnsupportedError();

  @override
  Future<void> close() async {}
}
