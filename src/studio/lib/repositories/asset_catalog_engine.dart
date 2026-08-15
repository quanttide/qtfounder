/// 资产目录引擎——读契约 → 遍历目录 → 解析命名 → 排序 → 构建目录树
library;

import 'dart:io';

import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/services.dart' show rootBundle;
import 'package:yaml/yaml.dart';

import '../models/asset_catalog.dart';

/// 引擎：加载契约 + 构建目录
class AssetCatalogEngine {
  final String contractAssetPath; // 打包路径（assets/contracts/xxx.yaml）
  final String? dataSourceRoot; // 数据源根（null = 用环境变量默认）

  AssetCatalogEngine({
    required this.contractAssetPath,
    this.dataSourceRoot,
  });

  /// 加载契约并构建目录
  Future<AssetCatalog> load() async {
    final contract = await _loadContract();
    _contractIgnore = contract.ignore;
    final rootPath = dataSourceRoot ?? _defaultRoot(contract.root);
    if (kIsWeb || rootPath == null || rootPath.isEmpty) {
      return AssetCatalog(contract: contract, nodes: const []);
    }
    final nodes = await _buildLevels(
      Directory(rootPath),
      contract.levels,
      0,
      contract.naming,
    );
    return AssetCatalog(contract: contract, nodes: nodes);
  }

  // ---- 契约加载 ----

  Future<AssetContract> _loadContract() async {
    final raw = await rootBundle.loadString(contractAssetPath);
    final yamlMap = loadYaml(raw) as YamlMap;
    return AssetContract(
      asset: yamlMap['asset'] as String,
      label: yamlMap['label'] as String,
      root: yamlMap['root'] as String,
      ignore: (yamlMap['ignore'] as YamlList? ?? const [])
          .cast<String>(),
      levels: (yamlMap['levels'] as YamlList)
          .map((l) => LevelSpec(
                key: (l as YamlMap)['key'] as String,
                label: l['label'] as String,
                optional: (l['optional'] as bool?) ?? false,
              ))
          .toList(),
      naming: _parseNaming(yamlMap['naming'] as YamlMap?),
    );
  }

  NamingSpec _parseNaming(YamlMap? m) {
    if (m == null) return const NamingSpec(unsorted: 'natural');
    return NamingSpec(
      pattern: m['pattern'] as String?,
      title: m['title'] as String?,
      sortKey: (m['sortKey'] as YamlList? ?? const []).cast<String>(),
      version: m['version'] as String?,
      unsorted: (m['unsorted'] as String?) ?? 'last',
    );
  }

  // ---- 目录遍历 ----

  Future<List<CatalogNode>> _buildLevels(
    Directory dir,
    List<LevelSpec> levels,
    int levelIndex,
    NamingSpec naming,
  ) async {
    if (levelIndex >= levels.length) {
      return const [];
    }
    final spec = levels[levelIndex];
    final nodes = <CatalogNode>[];
    final children = await dir.list().toList();
    for (final e in children) {
      if (e is! Directory) continue;
      final name = e.path.split(Platform.pathSeparator).last;
      if (levelIndex == levels.length - 1) {
        // 最后一级目录：下面直接是文件
        final files = await _listFiles(Directory(e.path), naming);
        nodes.add(CatalogNode(name: name, label: spec.label, children: const [], files: files));
      } else {
        // 下一级是 optional 且当前目录无子目录 → 文件直接挂当前节点
        final nextOptional = levelIndex + 1 < levels.length && levels[levelIndex + 1].optional;
        final subEntries = await Directory(e.path).list().toList();
        final hasSubDirs = subEntries.any((c) => c is Directory);
        if (nextOptional && !hasSubDirs) {
          final files = await _listFiles(Directory(e.path), naming);
          nodes.add(CatalogNode(name: name, label: spec.label, children: const [], files: files));
        } else {
          final sub = await _buildLevels(Directory(e.path), levels, levelIndex + 1, naming);
          nodes.add(CatalogNode(name: name, label: spec.label, children: sub, files: const []));
        }
      }
    }
    nodes.sort((a, b) => a.name.compareTo(b.name));
    return nodes;
  }

  Future<List<CatalogFile>> _listFiles(Directory dir, NamingSpec naming) async {
    final files = <CatalogFile>[];
    await for (final e in dir.list()) {
      if (e is! File || !e.path.endsWith('.md')) continue;
      final name = e.path.split(Platform.pathSeparator).last;
      if (_contractIgnore.contains(name)) continue;
      files.add(_parseFile(name, e.path, naming));
    }
    _sortFiles(files, naming);
    return files;
  }

  // ---- 命名解析 ----

  CatalogFile _parseFile(String fileName, String path, NamingSpec naming) {
    var title = fileName.replaceAll('.md', '');
    String? sortKey;
    var version = 1;
    final pattern = naming.pattern;
    if (pattern != null) {
      final re = RegExp(pattern);
      final match = re.firstMatch(fileName.replaceAll('.md', ''));
      if (match != null) {
        title = _group(match, naming.title) ?? title;
        if (naming.sortKey.isNotEmpty) {
          sortKey = naming.sortKey.map((g) => _group(match, g) ?? '').join('_');
        }
        if (naming.version != null) {
          version = int.tryParse(_group(match, naming.version) ?? '') ?? 1;
        }
      }
    }
    return CatalogFile(name: fileName, path: path, title: title, sortKey: sortKey, version: version);
  }

  String? _group(RegExpMatch match, String? ref) {
    if (ref == null) return null;
    final m = RegExp(r'^\$(\d+)$').firstMatch(ref);
    if (m == null) return ref;
    final idx = int.parse(m.group(1)!);
    return idx <= match.groupCount ? match.group(idx) : null;
  }

  // ---- 排序 ----

  void _sortFiles(List<CatalogFile> files, NamingSpec naming) {
    if (naming.unsorted == 'natural') {
      files.sort((a, b) => a.name.compareTo(b.name));
      return;
    }
    // sequence-first：已排序在前，未排序（sortKey==null）按名称在后
    final sorted = files.where((f) => f.sortKey != null).toList()
      ..sort((a, b) {
        final byKey = _compareSortKey(a.sortKey!, b.sortKey!);
        return byKey != 0 ? byKey : a.version.compareTo(b.version);
      });
    final unsorted = files.where((f) => f.sortKey == null).toList()
      ..sort((a, b) => a.name.compareTo(b.name));
    files
      ..clear()
      ..addAll(sorted)
      ..addAll(unsorted);
  }

  /// 排序键数值比较："1_1" < "10_1"（非字典序）
  int _compareSortKey(String a, String b) {
    final pa = a.split('_');
    final pb = b.split('_');
    for (var i = 0; i < pa.length && i < pb.length; i++) {
      final ia = int.tryParse(pa[i]) ?? 0;
      final ib = int.tryParse(pb[i]) ?? 0;
      if (ia != ib) return ia.compareTo(ib);
    }
    return a.compareTo(b);
  }

  // ---- 工具 ----

  List<String> _contractIgnore = const []; // 契约 ignore（加载时填充）

  String? _defaultRoot(String sub) {
    if (kIsWeb) return null;
    final home = Platform.environment['HOME'] ?? '';
    if (home.isEmpty) return null;
    return '$home/repos/quanttide-founder/assets/$sub';
  }
}
