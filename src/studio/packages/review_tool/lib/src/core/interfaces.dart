import 'article.dart';
import 'batch.dart';
import 'record.dart';
import 'types.dart';

typedef AiProbabilityResolver = double Function(
  String relativePath,
  String content,
);

abstract interface class ArticleSource {
  Future<List<Article>> scan({required String root, Set<String>? scope});

  Future<Article> read(String root, ArticleId id);
}

abstract interface class ReviewStore {
  Future<void> init();

  Future<ReviewRecord?> findExisting({
    required ArticleId articleId,
    required ContentHash originalHash,
  });

  Future<void> save(ReviewRecord record);

  Future<List<ReviewRecord>> history({int limit = 100});

  Future<BatchMeta?> batchMeta(BatchId batchId);

  Future<void> saveBatchMeta(BatchMeta meta);

  Future<List<BatchMeta>> pendingBatches();

  Future<Set<ArticleId>> reviewedArticleIds();

  Future<void> close();
}

abstract interface class DiffService {
  Future<String?> diff(String before, String after);
}

sealed class ReviewSubmitResult {
  const ReviewSubmitResult();
}

final class ReviewSubmitOk extends ReviewSubmitResult {
  const ReviewSubmitOk(this.record);

  final ReviewRecord record;
}

final class ReviewSubmitConflict extends ReviewSubmitResult {
  const ReviewSubmitConflict(this.message);

  final String message;
}

final class ReviewSubmitUnknownBatch extends ReviewSubmitResult {
  const ReviewSubmitUnknownBatch();
}

final class ReviewSubmitNotInBatch extends ReviewSubmitResult {
  const ReviewSubmitNotInBatch(this.articleId);

  final ArticleId articleId;
}
