import 'package:flutter_test/flutter_test.dart';
import 'package:review_tool/review_tool.dart';

void main() {
  test('review_tool 已接入 studio', () async {
    expect(
      hashOf('中文'),
      '72726d8818f693066ceb69afa364218b692e62ea92b385782363780f47529c21',
    );

    final diff = await const LineDiffService().diff('a\nb', 'a\nc');
    expect(diff, contains('- b'));
    expect(diff, contains('+ c'));
  });
}
