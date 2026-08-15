/// ThinkBloc 测试：日志列表、四分类分析缓存、忽略、采纳追加（原文叙事不变）。
library;

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as path;

import 'package:qtfounder_studio/bloc/think/think_bloc.dart';
import 'package:qtfounder_studio/models/analysis.dart';
import 'package:qtfounder_studio/models/emotion.dart';
import 'package:qtfounder_studio/repositories/emotion_repository.dart';
import 'package:qtfounder_studio/services/llm_client.dart';

/// 可控 LLM：记录调用次数，返回固定四分类结果
class _FakeLLMClient implements LLMClient {
  @override
  LLMConfig get config => const LLMConfig(baseUrl: 'fake', model: 'fake-model');

  int callCount = 0;
  String? lastContent;

  @override
  Future<ChapterAnalysis> analyzeStructure({
    required String chapterId,
    required String chapterPath,
    required String stageId,
    required String content,
    required List<String> previousSuggestions,
  }) {
    throw UnimplementedError();
  }

  @override
  Future<EmotionAnalysis> structureEmotion({
    required String journalPath,
    required String date,
    required String content,
  }) async {
    callCount++;
    lastContent = content;
    return EmotionAnalysis(
      journalPath: journalPath,
      date: date,
      entries: const [
        EmotionEntry(category: EmotionCategory.fact, text: '尝试更新网站', startLine: 2, endLine: 3),
        EmotionEntry(category: EmotionCategory.feeling, text: '困惑，逻辑不透明', startLine: 4, endLine: 4),
        EmotionEntry(category: EmotionCategory.need, text: '前台和后台分开', startLine: 6, endLine: 6),
        EmotionEntry(category: EmotionCategory.action, text: '先休息', startLine: 8, endLine: 8),
      ],
      analyzedAt: DateTime(2026, 8, 15),
      model: 'fake-model',
    );
  }

  @override
  void close() {}
}

/// 临时 memory 目录：journal/default/2026-08-13.md + 2026-08-14.md
class _TempMemory {
  final Directory dir;
  final String journalDir;

  _TempMemory(this.dir, this.journalDir);

  static Future<_TempMemory> create() async {
    final dir = await Directory.systemTemp.createTemp('think_bloc_test_');
    final journalDir = path.join(dir.path, 'journal', 'default');
    await Directory(journalDir).create(recursive: true);
    await File(path.join(journalDir, '2026-08-14.md'))
        .writeAsString('今天吃完饭回来，在路上就想了一下。\n\n尝试更新了网站。\n\n先休息。\n');
    await File(path.join(journalDir, '2026-08-13.md')).writeAsString('昨天写了初稿。\n');
    return _TempMemory(dir, journalDir);
  }

  String journalPath(String date) => path.join(journalDir, '$date.md');

  Future<void> delete() => dir.delete(recursive: true);
}

void main() {
  late _TempMemory memory;

  setUp(() async {
    memory = await _TempMemory.create();
  });

  tearDown(() async {
    await memory.delete();
  });

  final original14 = '今天吃完饭回来，在路上就想了一下。\n\n尝试更新了网站。\n\n先休息。\n';

  test('加载日志列表：日期倒序，未处理标记', () async {
    final bloc = ThinkBloc(
      journalRepository: JournalRepository(memoryPath: memory.dir.path),
      analysisRepository: EmotionAnalysisRepository(),
      llm: _FakeLLMClient(),
    );

    bloc.add(const LoadJournals());
    await bloc.stream.firstWhere((s) => s.journals.isNotEmpty);

    expect(bloc.state.journals.length, 2);
    expect(bloc.state.journals[0].date, '2026-08-14'); // 最新在前
    expect(bloc.state.journals[1].date, '2026-08-13');
    expect(bloc.state.journals.every((j) => !j.processed), isTrue);

    await bloc.close();
  });

  test('选择日志 → 读取内容；AI 整理 → 四分类落盘，原文逐字符不变', () async {
    final analysisRepo = EmotionAnalysisRepository();
    final llm = _FakeLLMClient();
    final bloc = ThinkBloc(
      journalRepository: JournalRepository(memoryPath: memory.dir.path),
      analysisRepository: analysisRepo,
      llm: llm,
    );

    bloc.add(const LoadJournals());
    await bloc.stream.firstWhere((s) => s.journals.isNotEmpty);

    bloc.add(const SelectJournal('2026-08-14'));
    await bloc.stream.firstWhere((s) => s.journalContent != null);
    expect(bloc.state.journalContent, original14);

    bloc.add(const AnalyzeJournal());
    await bloc.stream.firstWhere((s) => s.analysis != null && !s.isLoading);

    expect(llm.callCount, 1);
    final analysis = bloc.state.analysis!;
    expect(analysis.entries.length, 4);
    expect(analysis.byCategory(EmotionCategory.fact).length, 1);
    expect(analysis.byCategory(EmotionCategory.feeling).length, 1);
    expect(analysis.byCategory(EmotionCategory.need).length, 1);
    expect(analysis.byCategory(EmotionCategory.action).length, 1);
    expect(analysis.date, '2026-08-14');

    // 原文不变（核心验收）
    expect(await File(memory.journalPath('2026-08-14')).readAsString(), original14);

    // 缓存命中：重新选择 + 分析不重复调用 LLM
    final callsBefore = llm.callCount;
    bloc.add(const AnalyzeJournal());
    await Future<void>.delayed(const Duration(milliseconds: 100));
    expect(llm.callCount, callsBefore);

    // 缓存文件存在
    final cached = await analysisRepo.getAnalysis(memory.journalPath('2026-08-14'), '2026-08-14');
    expect(cached, isNotNull);
    expect(cached!.entries.length, 4);

    await bloc.close();
  });

  test('忽略条目：从结果移除并持久化', () async {
    final bloc = ThinkBloc(
      journalRepository: JournalRepository(memoryPath: memory.dir.path),
      analysisRepository: EmotionAnalysisRepository(),
      llm: _FakeLLMClient(),
    );

    bloc.add(const LoadJournals());
    await bloc.stream.firstWhere((s) => s.journals.isNotEmpty);
    bloc.add(const SelectJournal('2026-08-14'));
    await bloc.stream.firstWhere((s) => s.journalContent != null);
    bloc.add(const AnalyzeJournal());
    await bloc.stream.firstWhere((s) => s.analysis != null && !s.isLoading);

    final feeling = bloc.state.analysis!.byCategory(EmotionCategory.feeling).first;
    bloc.add(DismissEntry(feeling));
    await bloc.stream.firstWhere((s) => s.analysis != null && s.analysis!.entries.length == 3);

    expect(bloc.state.analysis!.byCategory(EmotionCategory.feeling), isEmpty);

    // 持久化：重新选择仍被忽略（状态与缓存全等被 bloc 去重，无新 emit）
    bloc.add(const SelectJournal('2026-08-14'));
    await Future<void>.delayed(const Duration(milliseconds: 100));
    expect(bloc.state.analysis!.entries.length, 3);

    await bloc.close();
  });

  test('采纳到日志：追加结构化段，正文叙事零改动，列表标记已处理', () async {
    final bloc = ThinkBloc(
      journalRepository: JournalRepository(memoryPath: memory.dir.path),
      analysisRepository: EmotionAnalysisRepository(),
      llm: _FakeLLMClient(),
    );

    bloc.add(const LoadJournals());
    await bloc.stream.firstWhere((s) => s.journals.isNotEmpty);
    bloc.add(const SelectJournal('2026-08-14'));
    await bloc.stream.firstWhere((s) => s.journalContent != null);
    bloc.add(const AnalyzeJournal());
    await bloc.stream.firstWhere((s) => s.analysis != null && !s.isLoading);

    bloc.add(const AdoptStructure());
    await bloc.stream.firstWhere((s) => !s.isAdopting && s.analysis != null);

    // 原文叙事仍在，结构化段追加在末尾
    final content = await File(memory.journalPath('2026-08-14')).readAsString();
    expect(content, contains(original14));
    expect(content, contains('## 结构化（思考云）'));
    expect(content, contains('事实：'));
    expect(content, contains('感受：'));
    expect(content, contains('需要：'));
    expect(content, contains('行动：'));
    expect(content, contains('- 尝试更新网站'));

    // 列表标记已处理
    final journal = bloc.state.journals.firstWhere((j) => j.date == '2026-08-14');
    expect(journal.processed, isTrue);

    await bloc.close();
  });
}
