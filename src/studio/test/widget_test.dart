import 'package:flutter_test/flutter_test.dart';

import 'package:qtfounder_studio/main.dart';

void main() {
  testWidgets('量潮创始人工作台可渲染创作现场', (WidgetTester tester) async {
    await tester.pumpWidget(const FounderApp());
    expect(find.text('量潮创始人工作台'), findsOneWidget);
    expect(find.text('创作流时间线'), findsOneWidget);
    expect(find.text('创作域矩阵'), findsOneWidget);
    expect(find.text('改稿轨迹'), findsOneWidget);
  });
}
