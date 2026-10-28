import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:qtfounder_studio/asset/repos/asset_repository.dart';
import 'package:qtfounder_studio/asset/models/asset_file.dart';

void main() {
  late Directory root;
  late AssetRepository repository;

  setUp(() async {
    root = await Directory.systemTemp.createTemp('asset_repo_');
    // scan 要求根目录有 .gitmodules。
    File(p.join(root.path, '.gitmodules')).writeAsStringSync('');
    repository = AssetRepository(
      root: root,
      trashDir: Directory(p.join(root.path, '.trash')),
    );
  });

  tearDown(() async {
    if (await root.exists()) await root.delete(recursive: true);
  });

  File write(String relativePath, [String content = 'x']) {
    final file = File(p.join(root.path, relativePath));
    file.parent.createSync(recursive: true);
    return file..writeAsStringSync(content);
  }

  group('scan', () {
    test('收文档与代码，按相对路径排序', () async {
      write('README.md', '# readme');
      write('src/app.dart', 'void main() {}');
      write('src/api.ts', 'export {}');
      write('docs/guide.md', 'guide');

      final assets = await repository.scan();

      expect(assets.map((asset) => asset.id), [
        'README.md',
        'docs/guide.md',
        'src/api.ts',
        'src/app.dart',
      ]);
      expect(assets.map((asset) => asset.kind), [
        AssetKind.document,
        AssetKind.document,
        AssetKind.code,
        AssetKind.code,
      ]);
    });

    test('忽略非目标后缀与排除目录', () async {
      write('keep.md');
      write('logo.png');
      write('data.json');
      write('build/out.md');
      write('node_modules/pkg/index.js');
      write('dist/bundle.js');
      write('.trash/old.md');

      final assets = await repository.scan();

      expect(assets.map((asset) => asset.id), ['keep.md']);
    });

    test('后缀大小写不敏感', () async {
      write('README.MD', '# readme');
      write('src/APP.DART', 'void main() {}');

      final assets = await repository.scan();

      expect(assets.map((asset) => asset.id), ['README.MD', 'src/APP.DART']);
    });

    test('根目录缺少 .gitmodules 时报错并带路径', () async {
      File(p.join(root.path, '.gitmodules')).deleteSync();

      expect(
        () => repository.scan(),
        throwsA(
          isA<AssetScanException>().having(
            (error) => error.message,
            'message',
            contains(root.path),
          ),
        ),
      );
    });
  });

  group('preview', () {
    test('返回文件开头内容', () async {
      final file = write('a.md', '第一行\n第二行');

      final text = await repository.preview(
        AssetFile(
          id: 'a.md',
          path: file.path,
          kind: AssetKind.document,
          size: 1,
          modified: DateTime(2026),
        ),
      );

      expect(text, '第一行\n第二行');
    });

    test('超过行数上限时截断并标注', () async {
      final file = write(
        'big.md',
        List.generate(100, (i) => 'line $i').join('\n'),
      );

      final text = await repository.preview(
        AssetFile(
          id: 'big.md',
          path: file.path,
          kind: AssetKind.document,
          size: 1,
          modified: DateTime(2026),
        ),
        maxLines: 10,
      );

      expect(text, startsWith('line 0\n'));
      expect(text, endsWith('\n…'));
      expect(text.split('\n'), hasLength(11));
    });
  });

  group('moveToTrash', () {
    test('保留相对路径移入回收目录', () async {
      final file = write('docs/guide.md', 'content');
      final asset = AssetFile(
        id: 'docs/guide.md',
        path: file.path,
        kind: AssetKind.document,
        size: 7,
        modified: DateTime(2026),
      );

      final moved = await repository.moveToTrash(asset);

      expect(await File(moved).readAsString(), 'content');
      expect(await file.exists(), isFalse);
      expect(p.relative(moved, from: repository.trashDir.path), asset.id);
    });

    test('重名时加时间戳前缀，不覆盖已有文件', () async {
      Directory(
        p.join(repository.trashDir.path, 'docs'),
      ).createSync(recursive: true);
      final existing = File(p.join(repository.trashDir.path, 'docs', 'a.md'))
        ..writeAsStringSync('回收目录里已有的文件');
      final file = write('docs/a.md', '新文件');
      final asset = AssetFile(
        id: 'docs/a.md',
        path: file.path,
        kind: AssetKind.document,
        size: 3,
        modified: DateTime(2026),
      );

      final moved = await repository.moveToTrash(asset);

      expect(await existing.readAsString(), '回收目录里已有的文件');
      expect(await File(moved).readAsString(), '新文件');
      expect(p.basename(moved), isNot('a.md'));
      expect(p.basename(moved), endsWith('-a.md'));
    });
  });
}
