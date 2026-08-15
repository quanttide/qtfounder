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

    // 切到写作（Bloc 编辑器——验证页面挂载）
    await tester.tap(find.byTooltip('写作'));
    await tester.pump();
    // 写作页挂载：搜索框 + 空状态（选择章节提示）出现
    final created =
        find.text('搜索章节...').evaluate().isNotEmpty ||
            find.text('选择一个章节开始写作').evaluate().isNotEmpty;
    expect(created, isTrue);

    // 切到思考（占位页）
    await tester.tap(find.byTooltip('思考'));
    await tester.pumpAndSettle();
    expect(find.text('思绪结构化（思考云原型，规划中）'), findsOneWidget);
  });
}
