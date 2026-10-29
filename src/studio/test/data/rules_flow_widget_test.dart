import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:qtfounder_studio/data/repos/data_store.dart';
import 'package:qtfounder_studio/data/repos/journal_repository.dart';
import 'package:qtfounder_studio/data/screens/rules_page.dart';

/// 页面层只钉导航链路：空态 → 编辑器。
/// 校验、保存落盘、试标流转的行为在 editor_cubit_test 里钉。
void main() {
  late Directory root;
  late DataStore store;
  late JournalRepository journal;

  setUp(() async {
    root = await Directory.systemTemp.createTemp('data_ui_');
    store = DataStore(root: root);
    journal = JournalRepository(root: root);
    final file = File(p.join(root.path, 'data/journal/default/2026-07/2026-07-20.md'));
    file.parent.createSync(recursive: true);
    file.writeAsStringSync('第一段。\n\n第二段。\n\n第三段。\n');
  });

  tearDown(() async {
    if (await root.exists()) await root.delete(recursive: true);
  });

  /// 真实目录 I/O 在假时钟下挂住：pumpWidget 与等待都放 runAsync 里，
  /// 回调回到假区后再 pump 一帧渲染。
  Future<void> pumpApp(WidgetTester tester) async {
    await tester.runAsync(() async {
      await tester.pumpWidget(
        MaterialApp(home: DataRulesPage(store: store, journal: journal)),
      );
      await Future<void>.delayed(const Duration(milliseconds: 50));
    });
    await tester.pump();
  }

  testWidgets('空存储给空态，点「设计规则」进编辑器', (tester) async {
    await pumpApp(tester);
    expect(find.text('还没有标注规则'), findsOneWidget);

    await tester.tap(find.widgetWithText(FloatingActionButton, '设计规则'));
    await tester.pumpAndSettle();

    expect(find.text('标签集'), findsOneWidget);
    expect(find.text('机器名'), findsOneWidget);
    expect(find.text('试标 3 段：验规则'), findsOneWidget);
    // 「导出任务包」在折叠线下，惰性列表不 build，不断言它。
  });
}
