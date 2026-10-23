import 'article.dart';
import 'types.dart';

class SampledArticle {
  const SampledArticle(this.article, this.weight);

  final Article article;
  final double weight;
}

class SampleBatch {
  const SampleBatch({required this.id, required this.articles, this.seed});

  final BatchId id;
  final List<SampledArticle> articles;
  final int? seed;

  List<ArticleId> get articleIds =>
      articles.map((sampled) => sampled.article.id).toList(growable: false);

  int get total => articles.length;
}

class BatchMeta {
  BatchMeta({
    required this.id,
    required this.createdAt,
    required this.articleIds,
    this.reviewedIds = const {},
  });

  final BatchId id;
  final DateTime createdAt;
  final List<ArticleId> articleIds;
  final Set<ArticleId> reviewedIds;

  int get total => articleIds.length;

  /// 只统计属于本批次的已审文章，避免 reviewedIds 混入外部 id 后分母失真。
  int get reviewed => articleIds.where(reviewedIds.contains).length;

  bool get isComplete => reviewed >= total;

  BatchMeta copyWith({Set<ArticleId>? reviewedIds}) => BatchMeta(
        id: id,
        createdAt: createdAt,
        articleIds: articleIds,
        reviewedIds: reviewedIds ?? this.reviewedIds,
      );

  Map<String, dynamic> toJson() => {
        'id': id,
        'created_at': createdAt.millisecondsSinceEpoch,
        'article_ids': articleIds,
        'reviewed_ids': reviewedIds.toList(growable: false),
      };

  factory BatchMeta.fromJson(Map<String, dynamic> json) => BatchMeta(
        id: json['id'] as String,
        createdAt:
            DateTime.fromMillisecondsSinceEpoch(json['created_at'] as int),
        articleIds: List<ArticleId>.from(json['article_ids'] as List),
        reviewedIds: (json['reviewed_ids'] as List).cast<ArticleId>().toSet(),
      );
}
