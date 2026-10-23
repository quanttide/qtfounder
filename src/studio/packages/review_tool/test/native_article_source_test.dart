import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;
import 'package:review_tool/review_tool.dart';

void main() {
  late Directory temp;
  late Directory root;

  setUp(() async {
    temp = await Directory.systemTemp.createTemp('review_source_');
    root = Directory(p.join(temp.path, 'kb'));
    await root.create();

    await File(p.join(root.path, 'a.md')).writeAsString('# Title A\n\nbody');
    await Directory(p.join(root.path, 'sub')).create();
    await File(p.join(root.path, 'sub', 'b.md')).writeAsString('# Title B\n');
    await Directory(p.join(root.path, '.git')).create();
    await File(
      p.join(root.path, '.git', 'ignored.md'),
    ).writeAsString('ignored');
    await Directory(p.join(root.path, 'node_modules')).create();
    await File(
      p.join(root.path, 'node_modules', 'ignored.md'),
    ).writeAsString('ignored');
    await File(p.join(root.path, '.hidden.md')).writeAsString('ignored');
    await File(p.join(root.path, 'note.txt')).writeAsString('ignored');
  });

  tearDown(() async {
    if (await temp.exists()) await temp.delete(recursive: true);
  });

  test('scan 只收 Markdown，跳过忽略目录和隐藏文件', () async {
    final articles = await const FsArticleSource().scan(root: root.path);

    expect(articles.map((article) => article.id), ['a.md', 'sub/b.md']);
    expect(articles.first.title, 'Title A');
    expect(articles.last.title, 'Title B');
    expect(articles.first.hash, hashOf('# Title A\n\nbody'));
  });

  test('read 返回根目录内的文章', () async {
    final article = await const FsArticleSource().read(root.path, 'a.md');

    expect(article.id, 'a.md');
    expect(article.content, contains('body'));
    expect(article.hash, hashOf(article.content));
  });

  test('scope 按目录边界过滤', () async {
    final scoped = await const FsArticleSource().scan(
      root: root.path,
      scope: {'sub'},
    );
    expect(scoped.map((article) => article.id), ['sub/b.md']);

    final prefixOnly = await const FsArticleSource().scan(
      root: root.path,
      scope: {'s'},
    );
    expect(prefixOnly, isEmpty);
  });

  test('aiProbabilityOf 回填 AI 来源', () async {
    final source = FsArticleSource(
      aiProbabilityOf: (id, content) => id == 'a.md' ? 1 : 0,
    );
    final articles = await source.scan(root: root.path);
    final a = articles.firstWhere((article) => article.id == 'a.md');
    final b = articles.firstWhere((article) => article.id == 'sub/b.md');

    expect(a.aiProbability, 1);
    expect(b.aiProbability, 0);
  });

  test('根目录不存在时抛 ReviewPathError', () async {
    expect(
      () => const FsArticleSource().scan(root: p.join(temp.path, 'missing')),
      throwsA(isA<ReviewPathError>()),
    );
  });

  test('拒绝 ../ 逃逸', () async {
    expect(
      () => const FsArticleSource().read(root.path, '../escape.md'),
      throwsA(isA<ReviewPathError>()),
    );
  });

  test('拒绝符号链接逃逸', () async {
    if (Platform.isWindows) return;

    final outside = File(p.join(temp.path, 'outside.md'));
    await outside.writeAsString('secret');
    await Link(p.join(root.path, 'link.md')).create(outside.path);

    expect(
      () => const FsArticleSource().read(root.path, 'link.md'),
      throwsA(isA<ReviewPathError>()),
    );

    final scanned = await const FsArticleSource().scan(root: root.path);
    expect(scanned.map((article) => article.id), isNot(contains('link.md')));
  });
}
