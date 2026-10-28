import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as p;

import '../models/asset_file.dart';

/// 扫描仓库根的文档与代码，把删除的资产移入回收目录。
class AssetRepository {
  AssetRepository({required this.root, required this.trashDir});

  /// 仓库根目录。
  final Directory root;

  /// 回收目录，位于 `<root>/.trash/`，保留相对路径结构。
  final Directory trashDir;

  /// 文档后缀。
  static const Set<String> documentExtensions = {'.md'};

  /// 代码后缀。
  static const Set<String> codeExtensions = {
    '.dart',
    '.js',
    '.jsx',
    '.ts',
    '.tsx',
    '.go',
    '.rs',
    '.py',
    '.sh',
    '.html',
    '.css',
    '.c',
    '.h',
    '.cc',
    '.cpp',
    '.hpp',
    '.swift',
    '.kt',
    '.kts',
    '.java',
    '.rb',
    '.php',
    '.lua',
    '.sql',
    '.tf',
    '.vue',
    '.svelte',
  };

  /// 不进入扫描的目录名。
  static const Set<String> excludedDirNames = {
    '.git',
    '.dart_tool',
    '.idea',
    '.vscode',
    'build',
    'dist',
    'node_modules',
    'target',
    'coverage',
    'Pods',
    'DerivedData',
    'venv',
    '.venv',
    '__pycache__',
    '.trash',
  };

  /// 解析仓库根：优先 `--dart-define=QTFOUNDER_ROOT`，否则从当前目录向上
  /// 找同时有 `.gitmodules`、`apps/`、`data/` 的目录，最多八层。
  ///
  /// 找不到也照常返回当前目录，让 `scan()` 带着路径走进 [AssetScanException]。
  factory AssetRepository.resolve() {
    const override = String.fromEnvironment('QTFOUNDER_ROOT');
    if (override.isNotEmpty) {
      final root = Directory(override);
      return AssetRepository(
        root: root,
        trashDir: Directory(p.join(root.path, '.trash')),
      );
    }

    var dir = Directory.current;
    for (var depth = 0; depth < 8; depth++) {
      if (_isRepoRoot(dir)) {
        return AssetRepository(
          root: dir,
          trashDir: Directory(p.join(dir.path, '.trash')),
        );
      }
      final parent = dir.parent;
      if (parent.path == dir.path) break;
      dir = parent;
    }

    final cwd = Directory.current;
    return AssetRepository(
      root: cwd,
      trashDir: Directory(p.join(cwd.path, '.trash')),
    );
  }

  static bool _isRepoRoot(Directory dir) =>
      File(p.join(dir.path, '.gitmodules')).existsSync() &&
      Directory(p.join(dir.path, 'apps')).existsSync() &&
      Directory(p.join(dir.path, 'data')).existsSync();

  /// 列出扫描范围内的文件，按相对路径排序。
  Future<List<AssetFile>> scan() async {
    if (!await root.exists()) {
      throw AssetScanException('仓库根目录不存在：${root.path}');
    }
    if (!File(p.join(root.path, '.gitmodules')).existsSync()) {
      throw AssetScanException(
        '不是 quanttide-founder 仓库根（缺少 .gitmodules）：${root.path}',
      );
    }

    final files = <File>[];
    await _walk(root, files);

    final assets = <AssetFile>[];
    for (final file in files) {
      final stat = await file.stat();
      assets.add(
        AssetFile(
          id: p.relative(file.path, from: root.path),
          path: p.absolute(file.path),
          kind: _kindOf(file.path)!,
          size: stat.size,
          modified: stat.modified,
        ),
      );
    }
    assets.sort((a, b) => a.id.compareTo(b.id));
    return assets;
  }

  /// 深度优先遍历，跳过排除目录与非目标后缀。
  Future<void> _walk(Directory dir, List<File> out) async {
    final subdirs = <Directory>[];
    for (final entry in await dir.list(followLinks: false).toList()) {
      if (entry is Directory) {
        if (excludedDirNames.contains(p.basename(entry.path))) continue;
        subdirs.add(entry);
      } else if (entry is File && _kindOf(entry.path) != null) {
        out.add(entry);
      }
    }
    await Future.wait([for (final subdir in subdirs) _walk(subdir, out)]);
  }

  static AssetKind? _kindOf(String path) {
    final ext = p.extension(path).toLowerCase();
    if (documentExtensions.contains(ext)) return AssetKind.document;
    if (codeExtensions.contains(ext)) return AssetKind.code;
    return null;
  }

  /// 读取文件开头作为卡片预览，最多 [maxLines] 行、[maxBytes] 字节。
  Future<String> preview(
    AssetFile asset, {
    int maxLines = 40,
    int maxBytes = 4000,
  }) async {
    final raf = await File(asset.path).open();
    try {
      final bytes = await raf.read(maxBytes);
      final text = utf8.decode(bytes, allowMalformed: true);
      final lines = text.split('\n');
      final truncated = bytes.length == maxBytes || lines.length > maxLines;
      final shown = lines.length > maxLines
          ? lines.sublist(0, maxLines)
          : lines;
      final body = shown.join('\n');
      return truncated ? '$body\n…' : body;
    } finally {
      await raf.close();
    }
  }

  /// 把资产移入回收目录，保留相对路径，返回移入后的路径。
  ///
  /// 回收目录不存在时先创建；目标重名时加时间戳前缀；优先 `rename`，
  /// 跨设备失败时回退到复制、校验字节数后再删源。
  Future<String> moveToTrash(AssetFile asset) async {
    await trashDir.create(recursive: true);

    var target = File(p.join(trashDir.path, asset.id));
    await target.parent.create(recursive: true);
    if (await target.exists()) {
      final stamp = _timestamp(DateTime.now());
      final dir = target.parent.path;
      final name = p.basename(asset.id);
      target = File(p.join(dir, '$stamp-$name'));
      var attempt = 1;
      while (await target.exists()) {
        target = File(p.join(dir, '$stamp-$attempt-$name'));
        attempt++;
      }
    }

    final source = File(asset.path);
    try {
      await source.rename(target.path);
    } on FileSystemException {
      await source.copy(target.path);
      final copied = await target.length();
      final original = await source.length();
      if (copied != original) {
        throw FileSystemException(
          '复制校验失败（$copied / $original 字节）',
          target.path,
        );
      }
      await source.delete();
    }
    return target.path;
  }

  static String _timestamp(DateTime now) {
    String two(int value) => value.toString().padLeft(2, '0');
    final millis = now.millisecond.toString().padLeft(3, '0');
    return '${now.year}${two(now.month)}${two(now.day)}'
        '-${two(now.hour)}${two(now.minute)}${two(now.second)}-$millis';
  }
}

/// 扫描失败，消息直接给用户看。
class AssetScanException implements Exception {
  AssetScanException(this.message);

  final String message;

  @override
  String toString() => message;
}
