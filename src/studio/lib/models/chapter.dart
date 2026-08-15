import 'package:equatable/equatable.dart';

/// 章节模型
class Chapter extends Equatable {
  final String id;
  final String title;
  final String stageId;
  final String path;
  final DateTime createdAt;
  final DateTime updatedAt;
  final int wordCount;
  final bool isFavorite;
  final int sortOrder;

  const Chapter({
    required this.id,
    required this.title,
    required this.stageId,
    required this.path,
    required this.createdAt,
    required this.updatedAt,
    this.wordCount = 0,
    this.isFavorite = false,
    this.sortOrder = 0,
  });

  Chapter copyWith({
    String? id,
    String? title,
    String? stageId,
    String? path,
    DateTime? createdAt,
    DateTime? updatedAt,
    int? wordCount,
    bool? isFavorite,
    int? sortOrder,
  }) {
    return Chapter(
      id: id ?? this.id,
      title: title ?? this.title,
      stageId: stageId ?? this.stageId,
      path: path ?? this.path,
      createdAt: createdAt ?? this.createdAt,
      updatedAt: updatedAt ?? this.updatedAt,
      wordCount: wordCount ?? this.wordCount,
      isFavorite: isFavorite ?? this.isFavorite,
      sortOrder: sortOrder ?? this.sortOrder,
    );
  }

  @override
  List<Object?> get props => [
        id,
        title,
        stageId,
        path,
        createdAt,
        updatedAt,
        wordCount,
        isFavorite,
        sortOrder,
      ];
}