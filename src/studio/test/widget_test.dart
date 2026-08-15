import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:qtfounder_studio/main.dart';

void main() {
  testWidgets('量潮创始人工作台可渲染资产职能', (WidgetTester tester) async {
    await tester.pumpWidget(const FounderApp());
    await tester.pumpAndSettle();

    // 品牌 + 导航
    expect(find.text('量潮'), findsOneWidget);
    expect(find.text('资产'), findsWidgets);
    expect(find.text('小说'), findsOneWidget);
    expect(find.text('记忆'), findsOneWidget);
  });

  testWidgets('导航栏三职能可切换', (WidgetTester tester) async {
    await tester.pumpWidget(const FounderApp());
    await tester.pumpAndSettle();

    // 切到创作（流程视图，含真实 IO——验证页面挂载）
    await tester.tap(find.byTooltip('创作'));
    await tester.pump();
    // 创作页挂载：加载中（CircularProgressIndicator）或标题出现
    final created =
        find.text('创作').evaluate().isNotEmpty ||
            find.byType(CircularProgressIndicator).evaluate().isNotEmpty;
    expect(created, isTrue);

    // 切到情绪（占位页）
    await tester.tap(find.byTooltip('情绪'));
    await tester.pumpAndSettle();
    expect(find.text('情绪状态（cli 数据接入，规划中）'), findsOneWidget);
  });
}
