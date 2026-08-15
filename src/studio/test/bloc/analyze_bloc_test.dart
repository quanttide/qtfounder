/// AnalyzeBloc 测试：缓存复用、去重、负反馈、原文不可变。
library;

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as path;

import 'package:qtfounder_studio/bloc/analyze/analyze_bloc.dart';
import 'package:qtfounder_studio/models/analysis.dart';
import 'package:qtfounder_studio/models/chapter.dart';
import 'package:qtfounder_studio/repositories/analysis_repository.dart';
import 'package:qtfounder_studio/repositories/chapter_repository.dart';
import 'package:qtfounder_studio/services/llm_client.dart';

import 'mock_chapter_repository.dart';

/// 可控 LLM：记录调用次数，返回固定分析结果
class _FakeLLMClient implements LLMClient {
  @override
  LLMConfig get config => const LLMConfig(baseUrl: 'fake', model: 'fake-model');

  int callCount = 0;
  List<String> lastPreviousSuggestions = const [];

  @override
  Future<ChapterAnalysis> analyzeStructure({
    required String chapterId,
    required String chapterPath,
    required String content,
    required List<String> previousSuggestions,
  }) async {
    callCount++;
    lastPreviousSuggestions = previousSuggestions;
    return ChapterAnalysis(
      chapterId: chapterId,
      chapterPath: chapterPath,
      suggestedStageId: '4_改稿',
      tags: const ['测试标签'],
      summary: '测试摘要',
      splitPoints: const [SplitPoint(line: 2, reason: '视角切换')],
      analyzedAt: DateTime(2026, 8, 15),
      model: 'fake-model',
    );
  }

  @override
  void close() {}
}

/// 基于真实文件系统的仓库（临时目录）
class _TempFileRepository implements ChapterRepository {
  final Directory dir;
  late final Chapter _chapter;
  late final File _file;
  String content;

  _TempFileRepository({required this.dir, required this.content})
      : _file = File(path.join(dir.path, '3_初稿', '1_1_章节.md')),
        _chapter = Chapter(
          id: '1_1_章节',
          title: '章节',
          stageId: '3_初稿',
          path: path.join(dir.path, '3_初稿', '1_1_章节.md'),
          createdAt: DateTime(2026, 8, 1),
          updatedAt: DateTime(2026, 8, 1),
        ) {
    _file.parent.createSync(recursive: true);
    _file.writeAsStringSync(content);
  }

  @override
  Future<List<Chapter>> getChapters() async => [_chapter];

  @override
  Future<Chapter?> getChapter(String id) async =>
      id == _chapter.id ? _chapter : null;

  @override
  Future<String> getChapterContent(String id) async => _file.readAsString();

  @override
  Future<void> saveChapter(String id, String content) async =>
      _file.writeAsString(content);

  @override
  Future<Chapter> createChapter(String stageId, String title) async =>
      throw UnimplementedError();

  @override
  Future<void> deleteChapter(String id) async =>
      throw UnimplementedError();

  @override
  Future<void> moveChapter(String id, String targetStageId) async =>
      throw UnimplementedError();

  @override
  Future<void> renameChapter(String id, String newTitle) async =>
      throw UnimplementedError();
}

void main() {
  late Directory tempDir;
  late String originalContent;

  setUp(() async {
    tempDir = await Directory.systemTemp.createTemp('analyze_bloc_test_');
    originalContent = '# 章节\n\n第一段。\n\n第二段。\n';
  });

  tearDown(() async {
    await tempDir.delete(recursive: true);
  });

  test('分析流程：结果落盘、缓存复用不重复调用 LLM、原文不变', () async {
    final repo = _TempFileRepository(dir: tempDir, content: originalContent);
    final analysisRepo = FileAnalysisRepository();
    final llm = _FakeLLMClient();
    final bloc = AnalyzeBloc(
      chapterRepository: repo,
      analysisRepository: analysisRepo,
      llm: llm,
    );

    // 首次分析
    bloc.add(const AnalyzeChapter('1_1_章节'));
    await bloc.stream.firstWhere((s) => s.analysis != null);
    expect(llm.callCount, 1);
    expect(bloc.state.analysis!.suggestedStageId, '4_改稿');
    expect(bloc.state.analysis!.tags, ['测试标签']);

    // 原文逐字符不变（核心验收）
    expect(await repo.getChapterContent('1_1_章节'), originalContent);

    // 缓存命中：再次分析不调用 LLM（状态与当前相等被去重，无新 emit）
    final callsBefore = llm.callCount;
    bloc.add(const AnalyzeChapter('1_1_章节'));
    await Future<void>.delayed(const Duration(milliseconds: 100));
    expect(llm.callCount, callsBefore);

    // 强制刷新：重新调用 LLM（新对象 → emit 正常）
    bloc.add(const RefreshAnalysis('1_1_章节'));
    await bloc.stream.firstWhere((s) => s.analysis != null && !s.isLoading);
    expect(llm.callCount, callsBefore + 1);

    await bloc.close();
  });

  test('负反馈：忽略建议后，下次分析携带 ignoredSuggestions', () async {
    final repo = _TempFileRepository(dir: tempDir, content: originalContent);
    final analysisRepo = FileAnalysisRepository();
    final llm = _FakeLLMClient();
    final bloc = AnalyzeBloc(
      chapterRepository: repo,
      analysisRepository: analysisRepo,
      llm: llm,
    );

    bloc.add(const AnalyzeChapter('1_1_章节'));
    await bloc.stream.firstWhere((s) => s.analysis != null);

    // 忽略拆分建议
    bloc.add(const IgnoreSuggestion('split_point_2'));
    await bloc.stream.firstWhere((s) =>
        s.analysis!.ignoredSuggestions.contains('split_point_2'));

    // 强制刷新：LLM 应收到负反馈
    bloc.add(const RefreshAnalysis('1_1_章节'));
    await bloc.stream.firstWhere((s) => s.analysis != null && !s.isLoading);
    expect(llm.lastPreviousSuggestions, contains('split_point_2'));

    await bloc.close();
  });

  test('不存在的章节：报错但不崩溃', () async {
    final repo = MockChapterRepository();
    final analysisRepo = FileAnalysisRepository();
    final llm = _FakeLLMClient();
    final bloc = AnalyzeBloc(
      chapterRepository: repo,
      analysisRepository: analysisRepo,
      llm: llm,
    );

    bloc.add(const AnalyzeChapter('not_exist'));
    await bloc.stream.firstWhere((s) => s.error != null);
    expect(bloc.state.error, contains('章节不存在'));
    expect(bloc.state.isLoading, isFalse);

    await bloc.close();
  });
}
