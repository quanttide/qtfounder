import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:qtfounder_studio/data/models/rule.dart';
import 'package:qtfounder_studio/data/repos/data_store.dart';

Rule _rule(String id, String name) => Rule(
  id: id,
  name: name,
  source: const RuleSource(),
  labels: const [RuleLabel(name: 'fact', label: '事实')],
  batch: 2,
  at: '2026-10-29T08:00:00Z',
);

void main() {
  late Directory root;
  late DataStore store;

  setUp(() async {
    root = await Directory.systemTemp.createTemp('data_store_');
    store = DataStore(root: root);
  });

  tearDown(() async {
    if (await root.exists()) await root.delete(recursive: true);
  });

  test('根解析三档', () {
    expect(DataStore.rootFrom('/tmp/x', '/xdg', '/home/u'), '/tmp/x');
    expect(DataStore.rootFrom('', '/xdg', '/home/u'), p.join('/xdg', 'qtfounder'));
    expect(DataStore.rootFrom(null, null, '/home/u'), p.join('/home/u', '.local/share/qtfounder'));
  });

  test('规则存取一来回，列表空态', () async {
    expect(await store.rules(), isEmpty);

    await store.saveRule(_rule('rul_AAA1', '日志四分类'));
    await store.saveRule(_rule('rul_BBB2', '另一条'));
    // 同 id 覆盖
    await store.saveRule(_rule('rul_AAA1', '改名了'));

    final rules = await store.rules();
    expect(rules, hasLength(2));
    expect(rules.first.id, 'rul_AAA1', reason: '按 id 升序');
    expect(rules.first.name, '改名了');
  });

  test('任务包落盘字段齐全', () async {
    final rule = _rule('rul_AAA1', '日志四分类');
    final task = TaskPackage(
      id: 'tsk_AAA1',
      ruleId: rule.id,
      rule: rule,
      units: [const Unit(id: 'u1', domain: 'default', text: '一段')],
      at: '2026-10-29T08:00:00Z',
    );
    final path = await store.exportTask(task);

    final decoded =
        jsonDecode(File(path).readAsStringSync()) as Map<String, dynamic>;
    expect(decoded['rule_id'], 'rul_AAA1');
    expect(decoded['rule']['labels'][0]['name'], 'fact', reason: '规则快照在任务包里');
    expect(decoded['units'], hasLength(1));
    expect(await store.taskCount(), 1);
  });

  test('规则校验与 CLI 同一把尺子', () {
    final draft = _rule('rul_AAA1', 'x');
    expect(draft.invalid, isNull);

    expect(draft.copyWith(name: '  ').invalid, '规则名不能为空');
    expect(draft.copyWith(batch: 0).invalid, '批次至少 1');
    expect(draft.copyWith(labels: const []).invalid, '至少要有一个标签');

    const duplicate = RuleLabel(name: 'fact', label: '重复');
    expect(
      draft.copyWith(labels: [draft.labels.first, duplicate]).invalid,
      '标签机器名重复：fact',
    );
  });
}
