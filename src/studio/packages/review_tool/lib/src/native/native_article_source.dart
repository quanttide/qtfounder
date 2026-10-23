import 'dart:io';

import 'package:path/path.dart' as p;

import '../core/article.dart';
import '../core/errors.dart';
import '../core/hash.dart';
import '../core/interfaces.dart';
import '../core/types.dart';

/// 文件系统文章源。
///
/// 只读，不写回原文。默认把 [aiProbability] 置 0（未知）；需要 AI 优先抽样时，
/// 通过 [aiProbabilityOf] 接入 frontmatter、manifest 或生成时落下的元数据。
class FsArticleSource implements ArticleSource {
  const FsArticleSource({this.aiProbabilityOf});

  final AiProbabilityResolver? aiProbabilityOf;

  @override
  Future<List<Article>> scan({required String root, Set<String>? scope}) async {
    final rootDir = Directory(root);
    if (!await rootDir.exists()) {
      throw ReviewPathError('知识库根目录不存在: $root');
    }

    // 用词法根目录算相对路径：root 自身可能是符号链接，不能拿 realRoot 去和
    // Directory.list 返回的路径比较。
    final lexicalRoot = p.canonicalize(root);
    final articles = <Article>[];

    await for (final entity in rootDir.list(
      recursive: true,
      followLinks: false,
    )) {
      if (entity is! File) continue;

      final relative = _relativeIfInside(lexicalRoot, entity.path);
      if (relative == null) continue;
      if (!relative.toLowerCase().endsWith('.md')) continue;
      if (_isIgnored(relative)) continue;
      if (!_matchesScope(relative, scope)) continue;

      try {
        final content = await entity.readAsString();
        final stat = await entity.stat();
        articles.add(
          Article(
            id: relative,
            absolutePath: entity.path,
            title: _extractTitle(content, relative),
            content: content,
            hash: hashOf(content),
            lastModified: stat.modified,
            aiProbability: _resolveAiProbability(relative, content),
          ),
        );
      } on FileSystemException {
        // 单个文件读失败不阻塞整批；根目录错误已经在上面抛过。
      }
    }

    articles.sort((a, b) => a.id.compareTo(b.id));
    return articles;
  }

  @override
  Future<Article> read(String root, ArticleId id) async {
    final rootDir = Directory(root);
    if (!await rootDir.exists()) {
      throw ReviewPathError('知识库根目录不存在: $root');
    }

    final realRoot = await rootDir.resolveSymbolicLinks();
    final candidate = p.canonicalize(p.join(realRoot, id));
    if (!p.isWithin(realRoot, candidate)) {
      throw ReviewPathError('文章 id 指向根目录外: $id');
    }

    final file = File(candidate);
    if (!await file.exists()) {
      throw ReviewPathError('文章不存在: $id');
    }

    final realFile = await file.resolveSymbolicLinks();
    if (!p.isWithin(realRoot, realFile)) {
      throw ReviewPathError('文章符号链接指向根目录外: $id');
    }

    final content = await file.readAsString();
    final stat = await file.stat();

    return Article(
      id: p.relative(realFile, from: realRoot).replaceAll(r'\', '/'),
      absolutePath: realFile,
      title: _extractTitle(content, id),
      content: content,
      hash: hashOf(content),
      lastModified: stat.modified,
      aiProbability: _resolveAiProbability(id, content),
    );
  }

  double _resolveAiProbability(String id, String content) {
    final value = aiProbabilityOf?.call(id, content) ?? 0;
    return value.clamp(0.0, 1.0);
  }

  static String? _relativeIfInside(String realRoot, String path) {
    final real = p.canonicalize(path);
    if (!p.isWithin(realRoot, real)) return null;
    return p.relative(real, from: realRoot).replaceAll(r'\', '/');
  }

  static bool _isIgnored(String relative) {
    return relative.split('/').any(
          (segment) =>
              segment == '.git' ||
              segment == '.dart_tool' ||
              segment == 'node_modules' ||
              segment == 'build' ||
              segment.startsWith('.'),
        );
  }

  static bool _matchesScope(String id, Set<String>? scope) {
    if (scope == null || scope.isEmpty) return true;
    return scope.any((entry) {
      final normalized = entry.replaceAll(r'\', '/');
      return id == normalized || id.startsWith('$normalized/');
    });
  }

  static String _extractTitle(String content, String path) {
    final match = RegExp(r'^#\s+(.+)$', multiLine: true).firstMatch(content);
    if (match != null) return match.group(1)!.trim();

    final name = path.split('/').last;
    return name.endsWith('.md') ? name.substring(0, name.length - 3) : name;
  }
}
