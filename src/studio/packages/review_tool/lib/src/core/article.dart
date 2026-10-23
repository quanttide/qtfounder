import 'types.dart';

class Article {
  const Article({
    required this.id,
    required this.absolutePath,
    required this.title,
    required this.content,
    required this.hash,
    required this.lastModified,
    this.aiProbability = 1,
  });

  final ArticleId id;
  final String absolutePath;
  final String title;
  final String content;
  final ContentHash hash;
  final DateTime lastModified;

  /// 0 表示未知。没有可信来源时不要默认归为“非 AI”。
  final double aiProbability;

  Article copyWith({
    ArticleId? id,
    String? absolutePath,
    String? title,
    String? content,
    ContentHash? hash,
    DateTime? lastModified,
    double? aiProbability,
  }) {
    return Article(
      id: id ?? this.id,
      absolutePath: absolutePath ?? this.absolutePath,
      title: title ?? this.title,
      content: content ?? this.content,
      hash: hash ?? this.hash,
      lastModified: lastModified ?? this.lastModified,
      aiProbability: aiProbability ?? this.aiProbability,
    );
  }
}
