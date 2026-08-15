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

    // 切到创作（占位页）
    await tester.tap(find.byTooltip('创作'));
    await tester.pumpAndSettle();
    expect(find.text('写作/改稿工作流（规划中）'), findsOneWidget);

    // 切到情绪（占位页）
    await tester.tap(find.byTooltip('情绪'));
    await tester.pumpAndSettle();
    expect(find.text('情绪状态（cli 数据接入，规划中）'), findsOneWidget);
  });
}
