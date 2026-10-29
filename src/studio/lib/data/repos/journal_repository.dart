import 'dart:io';

import 'package:path/path.dart' as p;

import '../models/journal_segment.dart';

/// 读 journal 日志，切成段落供试标与任务包导出。
///
/// 本仓只管「有哪几段」；标注结果由标注端产出，经 CLI `qtfounder data
/// result import` 进 XDG 存储，不在这里落盘。
class JournalRepository {
  JournalRepository({required this.root});

  /// 仓库根目录。
  final Directory root;

  /// journal 根目录，位于 `<root>/data/journal/`。
  Directory get journalDir => Directory(p.join(root.path, 'data', 'journal'));

  /// 解析仓库根：优先 `--dart-define=QTFOUNDER_ROOT`，否则从当前目录向上
  /// 找同时有 `.gitmodules`、`apps/`、`data/` 的目录，最多八层。
  ///
  /// 找不到也照常返回当前目录，让 [segments] 带着路径走进
  /// [JournalScanException]。
  factory JournalRepository.resolve() {
    const override = String.fromEnvironment('QTFOUNDER_ROOT');
    if (override.isNotEmpty) {
      return JournalRepository(root: Directory(override));
    }

    var dir = Directory.current;
    for (var depth = 0; depth < 8; depth++) {
      if (_isRepoRoot(dir)) return JournalRepository(root: dir);
      final parent = dir.parent;
      if (parent.path == dir.path) break;
      dir = parent;
    }
    return JournalRepository(root: Directory.current);
  }

  static bool _isRepoRoot(Directory dir) =>
      File(p.join(dir.path, '.gitmodules')).existsSync() &&
      Directory(p.join(dir.path, 'apps')).existsSync() &&
      Directory(p.join(dir.path, 'data')).existsSync();

  /// 全部 journal 段落：按文件（日期升序）再按段号排列。
  Future<List<JournalSegment>> segments() async {
    if (!await journalDir.exists()) {
      throw JournalScanException('journal 目录不存在：${journalDir.path}');
    }

    final files = <File>[];
    await _walk(journalDir, files);
    files.sort((a, b) => a.path.compareTo(b.path));

    final out = <JournalSegment>[];
    for (final file in files) {
      final text = await file.readAsString();
      final domain = p.basename(p.dirname(p.dirname(file.path)));
      var index = 0;
      for (final chunk in splitSegments(text)) {
        index++;
        out.add(
          JournalSegment(
            id: '${p.relative(file.path, from: root.path)}#$index',
            path: p.absolute(file.path),
            domain: domain,
            text: chunk,
          ),
        );
      }
    }
    return out;
  }

  /// 只收 `data/journal/**/*.md`，跳过隐藏目录。
  Future<void> _walk(Directory dir, List<File> out) async {
    final subdirs = <Directory>[];
    for (final entry in await dir.list(followLinks: false).toList()) {
      if (entry is Directory) {
        if (p.basename(entry.path).startsWith('.')) continue;
        subdirs.add(entry);
      } else if (entry is File && p.extension(entry.path) == '.md') {
        out.add(entry);
      }
    }
    await Future.wait([for (final subdir in subdirs) _walk(subdir, out)]);
  }

  /// 按空行与 `---` 分隔线切段；空段（纯标题、纯分隔线）丢弃。
  static List<String> splitSegments(String text) {
    final normalized = text.replaceAll('\r\n', '\n');
    final chunks = normalized.split(RegExp(r'\n-{3,}\n|\n{2,}'));
    return [
      for (final chunk in chunks)
        if (_meaningful(chunk)) chunk.trim(),
    ];
  }

  static bool _meaningful(String chunk) {
    final trimmed = chunk.trim();
    if (trimmed.isEmpty) return false;
    // 只有标题行（# 开头）或分隔线的块不算一段正文。
    final lines = trimmed
        .split('\n')
        .map((line) => line.trim())
        .where((line) => line.isNotEmpty)
        .toList();
    if (lines.isEmpty) return false;
    return lines.any((line) => !line.startsWith('#') && !line.startsWith('---'));
  }
}

/// 扫描失败，消息直接给用户看。
class JournalScanException implements Exception {
  JournalScanException(this.message);

  final String message;

  @override
  String toString() => message;
}
