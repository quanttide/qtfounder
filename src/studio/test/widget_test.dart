import 'package:flutter_test/flutter_test.dart';

import 'package:qtfounder_studio/main.dart';

void main() {
  testWidgets('导航壳可渲染，两个入口都在', (WidgetTester tester) async {
    await tester.pumpWidget(const FounderApp());

    expect(find.text('数据'), findsOneWidget);
    expect(find.text('资产'), findsOneWidget);
  });
}
