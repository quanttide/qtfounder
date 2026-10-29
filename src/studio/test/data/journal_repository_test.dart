import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:qtfounder_studio/data/repos/journal_repository.dart';

void main() {
  late Directory root;
  late JournalRepository repository;

  setUp(() async {
    root = await Directory.systemTemp.createTemp('journal_repo_');
    repository = JournalRepository(root: root);
  });

  tearDown(() async {
    if (await root.exists()) await root.delete(recursive: true);
  });

  void write(String relativePath, String content) {
    final file = File(p.join(root.path, relativePath));
    file.parent.createSync(recursive: true);
    file.writeAsStringSync(content);
  }

  test('空行与分隔线切段，标题段丢弃', () {
    final segments = JournalRepository.splitSegments('''
# 2026-10-13

第一段话。

第二段话。

---

第三段话。
''');
    expect(segments, ['第一段话。', '第二段话。', '第三段话。']);
  });

  test('segments 按文件与段号排列，域取自目录', () async {
    write(
      'data/journal/write/2026-10/2026-10-13.md',
      '写作用的一段。\n\n写作第二段。\n',
    );
    write('data/journal/default/2026-07/2026-07-20.md', '默认域的一段。\n');

    final segments = await repository.segments();

    expect(segments, hasLength(3));
    expect(segments[0].id, 'data/journal/default/2026-07/2026-07-20.md#1');
    expect(segments[0].domain, 'default');
    expect(segments[1].id, 'data/journal/write/2026-10/2026-10-13.md#1');
    expect(segments[2].domain, 'write');
  });

  test('journal 目录不存在时报错', () async {
    await expectLater(
      repository.segments(),
      throwsA(isA<JournalScanException>()),
    );
  });
}
