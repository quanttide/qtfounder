import 'types.dart';

enum ReviewOutcome { noIssue, fixed, partial, skipped }

enum TagCode {
  factualError,
  aiHallucination,
  logicJump,
  redundancy,
  style,
  other,
}

enum IssueSeverity { note, minor, major }

class IssueTag {
  const IssueTag({
    required this.code,
    this.severity = IssueSeverity.minor,
    this.startLine,
    this.endLine,
    this.anchorText,
    this.note,
  });

  final TagCode code;
  final IssueSeverity severity;
  final int? startLine;
  final int? endLine;
  final String? anchorText;
  final String? note;

  Map<String, dynamic> toJson() => {
        'c': code.name,
        's': severity.name,
        if (startLine != null) 'sl': startLine,
        if (endLine != null) 'el': endLine,
        if (anchorText != null) 'a': anchorText,
        if (note != null) 'n': note,
      };

  factory IssueTag.fromJson(Map<String, dynamic> json) => IssueTag(
        code: TagCode.values.byName(json['c'] as String),
        severity: IssueSeverity.values.byName(
          json['s'] as String? ?? IssueSeverity.minor.name,
        ),
        startLine: json['sl'] as int?,
        endLine: json['el'] as int?,
        anchorText: json['a'] as String?,
        note: json['n'] as String?,
      );
}

class ReviewRecord {
  ReviewRecord({
    required this.id,
    required this.batchId,
    required this.articleId,
    required this.originalHash,
    required this.revisedHash,
    this.baseCommit,
    required this.tags,
    required this.outcome,
    required this.activeMs,
    DateTime? submittedAt,
  }) : submittedAt = submittedAt ?? DateTime.now();

  final String id;
  final BatchId batchId;
  final ArticleId articleId;
  final ContentHash originalHash;
  final ContentHash revisedHash;
  final String? baseCommit;
  final List<IssueTag> tags;
  final ReviewOutcome outcome;
  final int activeMs;
  final DateTime submittedAt;

  ReviewRecord copyWith({
    String? id,
    BatchId? batchId,
    ArticleId? articleId,
    ContentHash? originalHash,
    ContentHash? revisedHash,
    String? baseCommit,
    List<IssueTag>? tags,
    ReviewOutcome? outcome,
    int? activeMs,
    DateTime? submittedAt,
  }) {
    return ReviewRecord(
      id: id ?? this.id,
      batchId: batchId ?? this.batchId,
      articleId: articleId ?? this.articleId,
      originalHash: originalHash ?? this.originalHash,
      revisedHash: revisedHash ?? this.revisedHash,
      baseCommit: baseCommit ?? this.baseCommit,
      tags: tags ?? this.tags,
      outcome: outcome ?? this.outcome,
      activeMs: activeMs ?? this.activeMs,
      submittedAt: submittedAt ?? this.submittedAt,
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'batch_id': batchId,
        'article_id': articleId,
        'original_hash': originalHash,
        'revised_hash': revisedHash,
        if (baseCommit != null) 'base_commit': baseCommit,
        'tags': tags.map((tag) => tag.toJson()).toList(),
        'outcome': outcome.name,
        'active_ms': activeMs,
        'submitted_at': submittedAt.millisecondsSinceEpoch,
      };

  factory ReviewRecord.fromJson(Map<String, dynamic> json) => ReviewRecord(
        id: json['id'] as String,
        batchId: json['batch_id'] as String,
        articleId: json['article_id'] as String,
        originalHash: json['original_hash'] as String,
        revisedHash: json['revised_hash'] as String,
        baseCommit: json['base_commit'] as String?,
        tags: (json['tags'] as List<dynamic>)
            .map((tag) => IssueTag.fromJson(tag as Map<String, dynamic>))
            .toList(),
        outcome: ReviewOutcome.values.byName(json['outcome'] as String),
        activeMs: json['active_ms'] as int,
        submittedAt: DateTime.fromMillisecondsSinceEpoch(
          json['submitted_at'] as int,
        ),
      );
}
