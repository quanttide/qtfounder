/// 思考层数据访问——日志仓库 + 结构化结果缓存。
///
/// 日志是输入（memory/journal/default/YYYY-MM-DD.md，兼容 journal/ 根布局），
/// 结构化结果缓存于日志同目录 `.analysis/<日期>.json`，删除即还原。
/// "采纳到日志"只在日志末尾追加 `## 结构化（思考云）` 段，不动正文叙事。
library;

import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as path;

import '../models/emotion.dart';

/// 日志条目（列表展示用）
class JournalEntry {
  final String date; // YYYY-MM-DD
  final String path; // 文件路径
  final bool processed; // 是否已结构化（含 ## 结构化 段）

  const JournalEntry({
    required this.date,
    required this.path,
    required this.processed,
  });
}

/// 日志仓库：列表 / 读取 / 追加结构化段
class JournalRepository {
  final String memoryPath;

  JournalRepository({required this.memoryPath});

  String get _journalDir {
    final defaultDir = path.join(memoryPath, 'journal', 'default');
    if (Directory(defaultDir).existsSync()) return defaultDir;
    return path.join(memoryPath, 'journal');
  }

  /// 列出所有日志（按日期倒序）
  Future<List<JournalEntry>> listJournals() async {
    final dir = Directory(_journalDir);
    if (!await dir.exists()) return const [];

    final entries = <JournalEntry>[];
    await for (final e in dir.list()) {
      if (e is! File || !e.path.endsWith('.md')) continue;
      final name = path.basenameWithoutExtension(e.path);
      // 仅识别 YYYY-MM-DD 格式
      if (!RegExp(r'^\d{4}-\d{2}-\d{2}$').hasMatch(name)) continue;
      final content = await e.readAsString();
      entries.add(JournalEntry(
        date: name,
        path: e.path,
        processed: content.contains('## 结构化'),
      ));
    }
    entries.sort((a, b) => b.date.compareTo(a.date)); // 最新在前
    return entries;
  }

  /// 读取日志全文
  Future<String> readJournal(String filePath) => File(filePath).readAsString();

  /// 追加结构化段到日志末尾（正文叙事零改动）
  Future<void> appendStructureSection(String filePath, EmotionAnalysis analysis) async {
    final file = File(filePath);
    final existing = await file.exists() ? await file.readAsString() : '';
    final section = _buildSection(analysis);
    final content = existing.trimRight().isEmpty
        ? section
        : '${existing.trimRight()}\n\n$section\n';
    await file.writeAsString(content);
  }

  String _buildSection(EmotionAnalysis analysis) {
    final buffer = StringBuffer('## 结构化（思考云）\n');
    for (final category in EmotionCategory.values) {
      final entries = analysis.byCategory(category);
      buffer.writeln('${category.label}：');
      if (entries.isEmpty) {
        buffer.writeln('- （无）');
        continue;
      }
      for (final e in entries) {
        buffer.writeln('- ${e.text}');
      }
    }
    return buffer.toString().trimRight();
  }
}

/// 结构化结果缓存——`.analysis/<日期>.json`，与日志物理分离
class EmotionAnalysisRepository {
  /// 缓存路径：日志同目录 .analysis/<日期>.json
  String _analysisPath(String journalPath, String date) {
    final dir = path.dirname(journalPath);
    return path.join(dir, '.analysis', '$date.json');
  }

  Future<EmotionAnalysis?> getAnalysis(String journalPath, String date) async {
    final file = File(_analysisPath(journalPath, date));
    if (!await file.exists()) return null;
    try {
      final json = jsonDecode(await file.readAsString()) as Map<String, dynamic>;
      return EmotionAnalysis.fromJson(json);
    } catch (_) {
      return null; // 损坏的缓存按不存在处理
    }
  }

  Future<void> saveAnalysis(EmotionAnalysis analysis) async {
    final file = File(_analysisPath(analysis.journalPath, analysis.date));
    await file.parent.create(recursive: true);
    await file.writeAsString(jsonEncode(analysis.toJson()));
  }

  Future<void> deleteAnalysis(String journalPath, String date) async {
    final file = File(_analysisPath(journalPath, date));
    if (await file.exists()) {
      await file.delete();
    }
  }
}
