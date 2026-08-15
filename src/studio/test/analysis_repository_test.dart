/// 整理层测试：分析仓库读写 + 原文不可变验收。
///
/// 核心验收（create.md）：任意 AI 操作后原文逐字符不变；
/// 删除 .analysis/ 目录即回到纯文本时代。
library;

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as path;

import 'package:qtfounder_studio/models/analysis.dart';
import 'package:qtfounder_studio/repositories/analysis_repository.dart';

void main() {
  late Directory tempDir;
  late File originalFile;
  late String originalContent;

  setUp(() async {
    tempDir = await Directory.systemTemp.createTemp('analysis_test_');
    originalFile = File(path.join(tempDir.path, '3_初稿', '1_1_测试章节.md'));
    await originalFile.parent.create(recursive: true);
    originalContent = '# 测试章节\n\n这是第一段。\n\n这是第二段。\n';
    await originalFile.writeAsString(originalContent);
  });

  tearDown(() async {
    await tempDir.delete(recursive: true);
  });

  test('保存分析结果写入 .analysis/ 目录，原文不受影响', () async {
    final repo = FileAnalysisRepository();
    final analysis = ChapterAnalysis(
      chapterId: '1_1_测试章节',
      chapterPath: originalFile.path,
      suggestedStageId: '4_改稿',
      tags: const ['职场', '重逢'],
      summary: '章节摘要',
      splitPoints: const [SplitPoint(line: 3, reason: '视角切换')],
      scenes: const [
        Scene(startLine: 1, endLine: 2, characters: ['林远亭'], summary: '场景一'),
      ],
      analyzedAt: DateTime(2026, 8, 15),
      model: 'test-model',
    );

    await repo.saveAnalysis(analysis);

    // 分析文件存在于 .analysis/ 子目录
    final analysisFile = File(path.join(
        tempDir.path, '3_初稿', '.analysis', '1_1_测试章节.json'));
    expect(await analysisFile.exists(), isTrue);

    // 原文逐字符不变（核心验收）
    expect(await originalFile.readAsString(), originalContent);

    // 读取回解析正确
    final loaded = await repo.getAnalysis(originalFile.path);
    expect(loaded, isNotNull);
    expect(loaded!.chapterId, '1_1_测试章节');
    expect(loaded.suggestedStageId, '4_改稿');
    expect(loaded.tags, ['职场', '重逢']);
    expect(loaded.splitPoints.single.line, 3);
    expect(loaded.scenes.single.characters, ['林远亭']);
    expect(loaded.ignoredSuggestions, isEmpty);
  });

  test('分析结果携带负反馈：保存后读取保留 ignoredSuggestions', () async {
    final repo = FileAnalysisRepository();
    final analysis = ChapterAnalysis(
      chapterId: 'c1',
      chapterPath: originalFile.path,
      ignoredSuggestions: const ['suggestion_split_3'],
      analyzedAt: DateTime(2026, 8, 15),
      model: 'test-model',
    );

    await repo.saveAnalysis(analysis);
    final loaded = await repo.getAnalysis(originalFile.path);

    expect(loaded!.ignoredSuggestions, ['suggestion_split_3']);
  });

  test('不存在/损坏的分析文件返回 null，不抛异常', () async {
    final repo = FileAnalysisRepository();

    // 不存在
    expect(await repo.getAnalysis(originalFile.path), isNull);

    // 损坏
    final analysisFile = File(path.join(
        tempDir.path, '3_初稿', '.analysis', '1_1_测试章节.json'));
    await analysisFile.parent.create(recursive: true);
    await analysisFile.writeAsString('{not valid json');
    expect(await repo.getAnalysis(originalFile.path), isNull);

    // 删除分析文件后原文仍在
    await repo.deleteAnalysis(originalFile.path);
    expect(await originalFile.exists(), isTrue);
    expect(await originalFile.readAsString(), originalContent);
  });

  test('删除 .analysis/ 目录不影响原文（回到纯文本时代）', () async {
    final repo = FileAnalysisRepository();
    final analysis = ChapterAnalysis(
      chapterId: 'c1',
      chapterPath: originalFile.path,
      analyzedAt: DateTime(2026, 8, 15),
      model: 'test-model',
    );
    await repo.saveAnalysis(analysis);

    // 删除整个 .analysis 目录
    final analysisDir = Directory(
        path.join(tempDir.path, '3_初稿', '.analysis'));
    await analysisDir.delete(recursive: true);

    expect(await analysisDir.exists(), isFalse);
    expect(await originalFile.exists(), isTrue);
    expect(await originalFile.readAsString(), originalContent);
  });
}
