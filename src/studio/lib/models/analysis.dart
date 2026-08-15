/// 整理层模型——AI 对章节的结构化分析结果。
///
/// 核心原则：整理层只包含"关于文本的信息"（元数据），
/// 绝不包含改写后的文本。删除 .analysis/ 目录即可完全还原。
library;

import 'package:equatable/equatable.dart';

/// 拆分建议（引用原文行号，不改写原文）
class SplitPoint extends Equatable {
  final int line;
  final String reason;

  const SplitPoint({required this.line, required this.reason});

  factory SplitPoint.fromJson(Map<String, dynamic> json) => SplitPoint(
        line: (json['line'] as num?)?.toInt() ?? 0,
        reason: (json['reason'] as String?) ?? '',
      );

  Map<String, dynamic> toJson() => {'line': line, 'reason': reason};

  @override
  List<Object?> get props => [line, reason];
}

/// 场景（只读展示）
class Scene extends Equatable {
  final int startLine;
  final int endLine;
  final List<String> characters;
  final String summary;

  const Scene({
    required this.startLine,
    required this.endLine,
    required this.characters,
    required this.summary,
  });

  factory Scene.fromJson(Map<String, dynamic> json) => Scene(
        startLine: (json['start_line'] as num?)?.toInt() ?? 0,
        endLine: (json['end_line'] as num?)?.toInt() ?? 0,
        characters: ((json['characters'] as List?) ?? const [])
            .map((e) => e.toString())
            .toList(),
        summary: (json['summary'] as String?) ?? '',
      );

  Map<String, dynamic> toJson() => {
        'start_line': startLine,
        'end_line': endLine,
        'characters': characters,
        'summary': summary,
      };

  @override
  List<Object?> get props => [startLine, endLine, characters, summary];
}

/// 章节分析结果
class ChapterAnalysis extends Equatable {
  final String chapterId;
  final String chapterPath;
  final String? suggestedStageId; // 阶段归类建议
  final List<String> tags; // 标签
  final String summary; // 摘要（只读）
  final List<SplitPoint> splitPoints; // 拆分建议
  final List<Scene> scenes; // 场景时间线（只读）
  final List<String> relatedChapters; // 相似章节（只读）
  final List<String> ignoredSuggestions; // 负反馈
  final DateTime analyzedAt; // 可追溯
  final String model; // 生成模型

  const ChapterAnalysis({
    required this.chapterId,
    required this.chapterPath,
    this.suggestedStageId,
    this.tags = const [],
    this.summary = '',
    this.splitPoints = const [],
    this.scenes = const [],
    this.relatedChapters = const [],
    this.ignoredSuggestions = const [],
    required this.analyzedAt,
    required this.model,
  });

  ChapterAnalysis copyWith({
    String? chapterId,
    String? chapterPath,
    String? suggestedStageId,
    List<String>? tags,
    String? summary,
    List<SplitPoint>? splitPoints,
    List<Scene>? scenes,
    List<String>? relatedChapters,
    List<String>? ignoredSuggestions,
    DateTime? analyzedAt,
    String? model,
  }) {
    return ChapterAnalysis(
      chapterId: chapterId ?? this.chapterId,
      chapterPath: chapterPath ?? this.chapterPath,
      suggestedStageId: suggestedStageId ?? this.suggestedStageId,
      tags: tags ?? this.tags,
      summary: summary ?? this.summary,
      splitPoints: splitPoints ?? this.splitPoints,
      scenes: scenes ?? this.scenes,
      relatedChapters: relatedChapters ?? this.relatedChapters,
      ignoredSuggestions: ignoredSuggestions ?? this.ignoredSuggestions,
      analyzedAt: analyzedAt ?? this.analyzedAt,
      model: model ?? this.model,
    );
  }

  factory ChapterAnalysis.fromJson(Map<String, dynamic> json) => ChapterAnalysis(
        chapterId: (json['chapter_id'] as String?) ?? '',
        chapterPath: (json['chapter_path'] as String?) ?? '',
        suggestedStageId: json['suggested_stage_id'] as String?,
        tags: ((json['tags'] as List?) ?? const []).map((e) => e.toString()).toList(),
        summary: (json['summary'] as String?) ?? '',
        splitPoints: ((json['split_points'] as List?) ?? const [])
            .map((e) => SplitPoint.fromJson(e as Map<String, dynamic>))
            .toList(),
        scenes: ((json['scenes'] as List?) ?? const [])
            .map((e) => Scene.fromJson(e as Map<String, dynamic>))
            .toList(),
        relatedChapters: ((json['related_chapters'] as List?) ?? const [])
            .map((e) => e.toString())
            .toList(),
        ignoredSuggestions: ((json['ignored_suggestions'] as List?) ?? const [])
            .map((e) => e.toString())
            .toList(),
        analyzedAt:
            DateTime.tryParse((json['analyzed_at'] as String?) ?? '') ?? DateTime.now(),
        model: (json['model'] as String?) ?? '',
      );

  Map<String, dynamic> toJson() => {
        'chapter_id': chapterId,
        'chapter_path': chapterPath,
        'suggested_stage_id': suggestedStageId,
        'tags': tags,
        'summary': summary,
        'split_points': splitPoints.map((e) => e.toJson()).toList(),
        'scenes': scenes.map((e) => e.toJson()).toList(),
        'related_chapters': relatedChapters,
        'ignored_suggestions': ignoredSuggestions,
        'analyzed_at': analyzedAt.toIso8601String(),
        'model': model,
      };

  @override
  List<Object?> get props => [
        chapterId,
        chapterPath,
        suggestedStageId,
        tags,
        summary,
        splitPoints,
        scenes,
        relatedChapters,
        ignoredSuggestions,
        analyzedAt,
        model,
      ];
}
