/// 整理层数据访问——与原文文件系统完全分离。
///
/// 分析结果存储于章节同目录的 `.analysis/<章节文件名>.json`，
/// 删除 `.analysis/` 目录即回到纯文本时代，原文零影响。
library;

import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as path;

import '../models/analysis.dart';

/// 整理层仓库抽象
abstract class ChapterAnalysisRepository {
  /// 读取章节分析结果（不存在返回 null）
  Future<ChapterAnalysis?> getAnalysis(String chapterPath);

  /// 保存分析结果
  Future<void> saveAnalysis(ChapterAnalysis analysis);

  /// 删除分析结果
  Future<void> deleteAnalysis(String chapterPath);
}

/// 基于文件系统的整理层实现
class FileAnalysisRepository implements ChapterAnalysisRepository {
  FileAnalysisRepository();

  /// 分析文件路径：<章节同目录>/.analysis/<文件名>.json
  String _analysisPath(String chapterPath) {
    final dir = path.dirname(chapterPath);
    final name = path.basenameWithoutExtension(chapterPath);
    return path.join(dir, '.analysis', '$name.json');
  }

  @override
  Future<ChapterAnalysis?> getAnalysis(String chapterPath) async {
    final file = File(_analysisPath(chapterPath));
    if (!await file.exists()) return null;
    try {
      final json = jsonDecode(await file.readAsString()) as Map<String, dynamic>;
      return ChapterAnalysis.fromJson(json);
    } catch (_) {
      return null; // 损坏的分析文件按不存在处理
    }
  }

  @override
  Future<void> saveAnalysis(ChapterAnalysis analysis) async {
    final file = File(_analysisPath(analysis.chapterPath));
    await file.parent.create(recursive: true);
    await file.writeAsString(jsonEncode(analysis.toJson()));
  }

  @override
  Future<void> deleteAnalysis(String chapterPath) async {
    final file = File(_analysisPath(chapterPath));
    if (await file.exists()) {
      await file.delete();
    }
  }
}
