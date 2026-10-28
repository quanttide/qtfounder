import 'dart:io';
import 'dart:math';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:qtfounder_studio/asset/bloc/asset_cubit.dart';
import 'package:qtfounder_studio/asset/data/asset_repository.dart';
import 'package:qtfounder_studio/asset/model/asset_file.dart';
import 'package:qtfounder_studio/asset/ui/asset_shell.dart';

void main() {
  late Directory root;
  late AssetRepository repository;
  late AssetCubit cubit;

  setUp(() async {
    root = await Directory.systemTemp.createTemp('asset_ui_');
    File(p.join(root.path, '.gitmodules')).writeAsStringSync('');
    _write(root, 'docs/a.md', 'a 的内容');
    _write(root, 'docs/b.md', 'b 的内容');
    _write(root, 'src/c.dart', 'c 的内容');
    repository = AssetRepository(
      root: root,
      trashDir: Directory(p.join(root.path, '.trash')),
    );
    cubit = AssetCubit(repository, random: Random(11));
  });

  tearDown(() async {
    await cubit.close();
    if (await root.exists()) await root.delete(recursive: true);
  });

  /// 真实目录 I/O 要放在 runAsync 里，否则 testWidgets 的假时钟会把它挂住。
  Future<void> pumpApp(WidgetTester tester) async {
    await tester.runAsync(cubit.load);
    await tester.pumpWidget(
      BlocProvider<AssetCubit>.value(
        value: cubit,
        child: const MaterialApp(home: AssetShell()),
      ),
    );
    await tester.pump();
  }

  testWidgets('删除与保留各一次后计数正确', (tester) async {
    await pumpApp(tester);
    expect(find.text('还剩 3 个 · 已标记删除 0'), findsOneWidget);

    await tester.tap(find.widgetWithText(FilledButton, '删除'));
    await tester.pump();
    expect(find.text('还剩 2 个 · 已标记删除 1'), findsOneWidget);

    await tester.tap(find.widgetWithText(FilledButton, '保留'));
    await tester.pump();
    expect(find.text('还剩 1 个 · 已标记删除 1'), findsOneWidget);
  });

  testWidgets('卡片展示文件路径与预览', (tester) async {
    await pumpApp(tester);

    final ready = cubit.state as AssetReady;
    expect(find.text(ready.asset.id), findsOneWidget);
    expect(find.text(ready.previews[ready.asset.id]!), findsOneWidget);
  });

  testWidgets('走完一批进确认页，确认后文件真的移走', (tester) async {
    await pumpApp(tester);
    await tester.tap(find.widgetWithText(FilledButton, '删除'));
    await tester.pump();
    await tester.tap(find.widgetWithText(FilledButton, '保留'));
    await tester.pump();
    await tester.tap(find.widgetWithText(FilledButton, '删除'));
    await tester.pump();

    final confirm = cubit.state as AssetConfirm;
    expect(find.text('将删除 ${confirm.marked.length} 个资产'), findsOneWidget);
    expect(confirm.marked, hasLength(2));

    await tester.runAsync(() async {
      await tester.tap(find.widgetWithText(FilledButton, '移入回收目录'));
      for (var i = 0; i < 100 && cubit.state is! AssetDone; i++) {
        await Future<void>.delayed(const Duration(milliseconds: 20));
      }
    });
    await tester.pump();

    final done = cubit.state as AssetDone;
    expect(find.text('已移入回收目录 ${done.succeeded.length} 个'), findsOneWidget);
    expect(done.succeeded, hasLength(2));
    expect(done.failed, isEmpty);

    // 标记删除的文件：原路径消失、回收目录里按相对路径出现。
    for (final asset in confirm.marked) {
      expect(File(asset.path).existsSync(), isFalse);
      expect(
        File(p.join(repository.trashDir.path, asset.id)).existsSync(),
        isTrue,
      );
    }
    // 保留的文件留在原地。
    final kept = confirm.assets.firstWhere(
      (asset) => confirm.decisions[asset.id] == Decision.keep,
    );
    expect(File(kept.path).existsSync(), isTrue);
  });
}

void _write(Directory root, String relativePath, String content) {
  final file = File(p.join(root.path, relativePath));
  file.parent.createSync(recursive: true);
  file.writeAsStringSync(content);
}
