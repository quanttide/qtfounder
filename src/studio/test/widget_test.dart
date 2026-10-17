import 'package:flutter_test/flutter_test.dart';

import 'package:qtfounder_studio/main.dart';

void main() {
  testWidgets('骨架可渲染', (WidgetTester tester) async {
    await tester.pumpWidget(const FounderApp());

    expect(find.text('量潮创始人工作台'), findsOneWidget);
  });
}
