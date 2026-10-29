import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:qtfounder_studio/data/repos/journal_repository.dart';
import 'package:qtfounder_studio/data/screens/data_shell.dart';
import 'package:qtfounder_studio/data/states/data_cubit.dart';

void main() {
  late Directory root;
  late JournalRepository repository;
  late DataCubit cubit;

  setUp(() async {
    root = await Directory.systemTemp.createTemp('data_ui_');
    _write(root, 'data/journal/write/2026-10/2026-10-13.md', '写作的一段。\n');
    _write(root, 'data/journal/think/2026-10/2026-10-17.md', '思考的一段。\n');
    repository = JournalRepository(root: root);
    cubit = DataCubit(repository);
  });

  tearDown(() async {
    await cubit.close();
    if (await root.exists()) await root.delete(recursive: true);
  });

  /// 真实目录 I/O 要放在 runAsync 里，否则 testWidgets 的假时钟会把它挂住。
  Future<void> pumpApp(WidgetTester tester) async {
    await tester.runAsync(cubit.load);
    await tester.pumpWidget(
      BlocProvider<DataCubit>.value(
        value: cubit,
        child: const MaterialApp(home: DataShell()),
      ),
    );
    await tester.pump();
  }

  testWidgets('卡片展示段落与域', (tester) async {
    await pumpApp(tester);

    final ready = cubit.state as DataReady;
    expect(find.text(ready.segment.text), findsOneWidget);
    expect(find.text(ready.segment.domain), findsOneWidget);
  });

  testWidgets('贴两个标签后进保存页，保存后文件里有两行', (tester) async {
    await pumpApp(tester);

    await tester.tap(find.widgetWithText(FilledButton, '事实'));
    await tester.pump();
    await tester.tap(find.widgetWithText(FilledButton, '感受'));
    await tester.pump();

    final save = cubit.state as DataSave;
    expect(find.text('本批标注 ${save.annotations.length} 段'), findsOneWidget);

    await tester.runAsync(() async {
      await tester.tap(find.widgetWithText(FilledButton, '保存标注'));
      for (var i = 0; i < 100 && cubit.state is! DataDone; i++) {
        await Future<void>.delayed(const Duration(milliseconds: 20));
      }
    });
    await tester.pump();

    final done = cubit.state as DataDone;
    expect(find.text('已保存 ${done.count} 条标注'), findsOneWidget);

    final lines = File(
      p.join(done.saved),
    ).readAsLinesSync();
    expect(lines, hasLength(done.count));
  });
}

void _write(Directory root, String relativePath, String content) {
  final file = File(p.join(root.path, relativePath));
  file.parent.createSync(recursive: true);
  file.writeAsStringSync(content);
}
