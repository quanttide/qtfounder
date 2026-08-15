/// 思考层模型——情绪结构化处理器的分析结果。
///
/// 核心原则（对齐 emotion-agent 设计）：不做判断、不替用户扛，
/// 只把混沌拆开——日志是输入，四分类条目是输出。
/// 每条目引用原文行号（元数据），绝不包含改写后的文本。
library;

import 'package:equatable/equatable.dart';

/// 情绪结构化分类
enum EmotionCategory {
  /// 事实（发生了什么）
  fact('事实', '发生了什么'),

  /// 感受（情绪如何）
  feeling('感受', '情绪如何'),

  /// 需要（真正想要什么）
  need('需要', '真正想要什么'),

  /// 行动（下一步行动）
  action('行动', '可以做什么');

  final String label;
  final String hint;

  const EmotionCategory(this.label, this.hint);

  static EmotionCategory fromName(String? name) {
    switch (name) {
      case 'feeling':
        return EmotionCategory.feeling;
      case 'need':
        return EmotionCategory.need;
      case 'action':
        return EmotionCategory.action;
      default:
        return EmotionCategory.fact;
    }
  }

  String get name => switch (this) {
        EmotionCategory.fact => 'fact',
        EmotionCategory.feeling => 'feeling',
        EmotionCategory.need => 'need',
        EmotionCategory.action => 'action',
      };
}

/// 结构化条目（引用原文行号，提炼使用原文词汇）
class EmotionEntry extends Equatable {
  final EmotionCategory category;
  final String text; // 一句话提炼（原文词汇）
  final int startLine; // 原文起始行（1 基）
  final int endLine; // 原文结束行

  const EmotionEntry({
    required this.category,
    required this.text,
    required this.startLine,
    required this.endLine,
  });

  factory EmotionEntry.fromJson(Map<String, dynamic> json) => EmotionEntry(
        category: EmotionCategory.fromName(json['category'] as String?),
        text: (json['text'] as String?) ?? '',
        startLine: (json['start_line'] as num?)?.toInt() ?? 0,
        endLine: (json['end_line'] as num?)?.toInt() ?? 0,
      );

  Map<String, dynamic> toJson() => {
        'category': category.name,
        'text': text,
        'start_line': startLine,
        'end_line': endLine,
      };

  @override
  List<Object?> get props => [category, text, startLine, endLine];
}

/// 某天日志的结构化分析结果
class EmotionAnalysis extends Equatable {
  final String journalPath; // 日志文件路径
  final String date; // YYYY-MM-DD
  final List<EmotionEntry> entries;
  final DateTime analyzedAt; // 可追溯
  final String model; // 生成模型

  const EmotionAnalysis({
    required this.journalPath,
    required this.date,
    this.entries = const [],
    required this.analyzedAt,
    required this.model,
  });

  /// 按分类过滤
  List<EmotionEntry> byCategory(EmotionCategory category) =>
      entries.where((e) => e.category == category).toList();

  EmotionAnalysis copyWith({
    String? journalPath,
    String? date,
    List<EmotionEntry>? entries,
    DateTime? analyzedAt,
    String? model,
  }) {
    return EmotionAnalysis(
      journalPath: journalPath ?? this.journalPath,
      date: date ?? this.date,
      entries: entries ?? this.entries,
      analyzedAt: analyzedAt ?? this.analyzedAt,
      model: model ?? this.model,
    );
  }

  factory EmotionAnalysis.fromJson(Map<String, dynamic> json) => EmotionAnalysis(
        journalPath: (json['journal_path'] as String?) ?? '',
        date: (json['date'] as String?) ?? '',
        entries: ((json['entries'] as List?) ?? const [])
            .map((e) => EmotionEntry.fromJson(e as Map<String, dynamic>))
            .toList(),
        analyzedAt:
            DateTime.tryParse((json['analyzed_at'] as String?) ?? '') ?? DateTime.now(),
        model: (json['model'] as String?) ?? '',
      );

  Map<String, dynamic> toJson() => {
        'journal_path': journalPath,
        'date': date,
        'entries': entries.map((e) => e.toJson()).toList(),
        'analyzed_at': analyzedAt.toIso8601String(),
        'model': model,
      };

  @override
  List<Object?> get props => [journalPath, date, entries, analyzedAt, model];
}
