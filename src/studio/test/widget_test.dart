import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:qtfounder_studio/data/creative_repository.dart';
import 'package:qtfounder_studio/main.dart';

void main() {
  testWidgets('量潮创始人工作台可渲染创作现场（注入数据）', (WidgetTester tester) async {
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: CreativeDesk(
            chaptersLoader: _fakeChapters,
            memoryLoader: _fakeMemory,
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    expect(find.textContaining('改稿轨迹'), findsOneWidget);
    expect(find.textContaining('创作域矩阵'), findsOneWidget);
    expect(find.text('1_1_咖啡厅重逢'), findsOneWidget);
    expect(find.text('[context] fiction-plot'), findsOneWidget);
  });
}

Future<List<CreativeItem>> _fakeChapters() async => const [
      CreativeItem(name: '1_1_咖啡厅重逢', path: 'test', category: '改稿'),
      CreativeItem(name: '7_1_酒吧表白', path: 'test', category: '改稿'),
    ];

Future<List<CreativeItem>> _fakeMemory() async => const [
      CreativeItem(name: 'fiction', path: 'test', category: 'roadmap'),
      CreativeItem(name: 'fiction-plot', path: 'test', category: 'context'),
    ];
