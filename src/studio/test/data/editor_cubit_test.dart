import 'dart:io';
import 'dart:math';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:qtfounder_studio/data/models/rule.dart';
import 'package:qtfounder_studio/data/repos/data_store.dart';
import 'package:qtfounder_studio/data/repos/journal_repository.dart';
import 'package:qtfounder_studio/data/states/editor_cubit.dart';
import 'package:qtfounder_studio/data/states/rules_cubit.dart';

Rule draft() => Rule(
  id: 'rul_AAA1',
  name: '日志四分类',
  source: const RuleSource(),
  labels: const [
    RuleLabel(name: 'fact', label: '事实'),
    RuleLabel(name: 'feeling', label: '感受'),
  ],
  batch: 2,
  at: '2026-10-29T08:00:00Z',
);

void main() {
  late Directory root;
  late DataStore store;
  late JournalRepository journal;

  setUp(() async {
    root = await Directory.systemTemp.createTemp('editor_');
    store = DataStore(root: root);
    journal = JournalRepository(root: root);
    final file = File(p.join(root.path, 'data/journal/default/2026-07/2026-07-20.md'));
    file.parent.createSync(recursive: true);
    file.writeAsStringSync('第一段。\n\n第二段。\n\n第三段。\n');
  });

  tearDown(() async {
    if (await root.exists()) await root.delete(recursive: true);
  });

  test('规则列表读存储，新草稿有标签占位', () async {
    final cubit = RulesCubit(store);
    await cubit.load();
    expect((cubit.state as RulesList).rules, isEmpty);

    final draft = cubit.newDraft();
    expect(draft.id, startsWith('rul_'));
    expect(draft.labels, hasLength(1), reason: '给一格空位起步');
    expect(draft.invalid, isNotNull, reason: '空草稿过不了校验');
    await cubit.close();
  });

  test('试标三段：贴两个标签进摘要', () async {
    final cubit = EditorCubit(store, journal, draft: draft(), random: Random(7));
    await cubit.startTrial();

    final trial = cubit.state.trial!;
    expect(trial.segments, hasLength(3));

    cubit.trialLabel('fact');
    cubit.trialSkip();
    cubit.trialLabel('feeling');

    final done = cubit.state.trial!;
    expect(done.done, isTrue);
    expect(done.labels, hasLength(2));
    expect(done.skipped, hasLength(1));

    cubit.endTrial();
    expect(cubit.state.trial, isNull);
    await cubit.close();
  });

  test('保存与导出落 XDG 存储', () async {
    final cubit = EditorCubit(store, journal, draft: draft(), random: Random(7));

    await cubit.save();
    expect(cubit.state.savedPath, isNotNull);
    expect(await store.rules(), hasLength(1));

    await cubit.exportTask();
    expect(cubit.state.exportedPath, isNotNull);
    expect(await store.taskCount(), 1);
    await cubit.close();
  });

  test('空草稿保存报错不落盘', () async {
    final cubit = EditorCubit(
      store,
      journal,
      draft: draft().copyWith(name: ''),
      random: Random(7),
    );
    await cubit.save();

    expect(cubit.state.error, '规则名不能为空');
    expect(cubit.state.savedPath, isNull);
    await cubit.close();
  });
}
