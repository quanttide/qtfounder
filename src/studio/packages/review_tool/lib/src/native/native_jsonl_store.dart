import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as p;

import '../core/batch.dart';
import '../core/interfaces.dart';
import '../core/record.dart';
import '../core/types.dart';

/// 追加式 JSONL 存储：`records.jsonl` + `batches.jsonl`。
///
/// 选 JSONL 而不是 SQLite：审阅记录量级小，真源可读可 diff；不引入原生
/// SQLite 依赖，桌面/移动端行为一致。后续需要查询再把它当源建索引。
class JsonlReviewStore implements ReviewStore {
  JsonlReviewStore({required this.directory});

  final String directory;

  String get _recordsPath => p.join(directory, 'records.jsonl');
  String get _batchesPath => p.join(directory, 'batches.jsonl');

  @override
  Future<void> init() async {
    await Directory(directory).create(recursive: true);
  }

  @override
  Future<ReviewRecord?> findExisting({
    required ArticleId articleId,
    required ContentHash originalHash,
  }) async {
    final records = await _readRecords();
    for (final record in records.reversed) {
      if (record.articleId == articleId &&
          record.originalHash == originalHash) {
        return record;
      }
    }
    return null;
  }

  @override
  Future<void> save(ReviewRecord record) async {
    await _append(_recordsPath, record.toJson());
  }

  @override
  Future<List<ReviewRecord>> history({int limit = 100}) async {
    final records = await _readRecords();
    records.sort((a, b) => b.submittedAt.compareTo(a.submittedAt));
    return records.take(limit).toList(growable: false);
  }

  @override
  Future<BatchMeta?> batchMeta(BatchId batchId) async {
    final batches = await _readBatches();
    for (final batch in batches.reversed) {
      if (batch.id == batchId) return batch;
    }
    return null;
  }

  @override
  Future<void> saveBatchMeta(BatchMeta meta) async {
    await _append(_batchesPath, meta.toJson());
  }

  @override
  Future<List<BatchMeta>> pendingBatches() async {
    final latest = <BatchId, BatchMeta>{};
    for (final batch in await _readBatches()) {
      latest[batch.id] = batch;
    }

    final pending = latest.values
        .where((batch) => !batch.isComplete)
        .toList(growable: false)
      ..sort((a, b) => a.createdAt.compareTo(b.createdAt));
    return pending;
  }

  @override
  Future<Set<ArticleId>> reviewedArticleIds() async {
    final records = await _readRecords();
    return records.map((record) => record.articleId).toSet();
  }

  @override
  Future<void> close() async {}

  Future<List<ReviewRecord>> _readRecords() async {
    final byId = <String, ReviewRecord>{};
    for (final row in await _readJsonLines(_recordsPath)) {
      try {
        final record = ReviewRecord.fromJson(row);
        byId[record.id] = record;
      } on Object {
        // 跳过损坏行，保留其余可读记录。
      }
    }
    return byId.values.toList(growable: false);
  }

  Future<List<BatchMeta>> _readBatches() async {
    final byId = <BatchId, BatchMeta>{};
    for (final row in await _readJsonLines(_batchesPath)) {
      try {
        final batch = BatchMeta.fromJson(row);
        byId[batch.id] = batch;
      } on Object {
        // 跳过损坏行，保留其余可读批次。
      }
    }
    return byId.values.toList(growable: false);
  }

  Future<List<Map<String, dynamic>>> _readJsonLines(String path) async {
    final file = File(path);
    if (!await file.exists()) return const [];

    final result = <Map<String, dynamic>>[];
    for (final line in await file.readAsLines()) {
      if (line.trim().isEmpty) continue;
      try {
        final decoded = jsonDecode(line);
        if (decoded is Map<String, dynamic>) result.add(decoded);
      } on FormatException {
        // 单行损坏不阻塞整份记录。
      }
    }
    return result;
  }

  Future<void> _append(String path, Map<String, dynamic> json) async {
    await Directory(directory).create(recursive: true);
    await File(path).writeAsString(
      '${jsonEncode(json)}\n',
      mode: FileMode.append,
      flush: true,
    );
  }
}
