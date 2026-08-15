/// 创作数据仓库——从 fiction / memory 数据源加载创作现场数据
///
/// 桌面端：直接读文件系统（dart:io）。
/// Web 端：无文件系统，返回内置示例数据。
library;

import 'dart:io';

import 'package:flutter/foundation.dart' show kIsWeb;

import '../config.dart';

/// 创作域条目
class CreativeItem {
  final String name;
  final String path;
  final String category;

  const CreativeItem({
    required this.name,
    required this.path,
    required this.category,
  });
}

/// 加载改稿章节列表（fiction/职场言情/4_改稿/*.md）
Future<List<CreativeItem>> loadRevisionChapters() async {
  if (kIsWeb || !dataSourceAvailable) return _fallbackChapters();
  final dir = Directory('$fictionPath/职场言情/4_改稿');
  if (!await dir.exists()) return _fallbackChapters();
  final files = await dir
      .list()
      .where((e) => e.path.endsWith('.md'))
      .toList();
  files.sort((a, b) => a.path.compareTo(b.path));
  return files
      .map((f) => CreativeItem(
            name: f.path.split('/').last.replaceAll('.md', ''),
            path: f.path,
            category: '改稿',
          ))
      .toList();
}

/// 加载 memory 文档列表（roadmap/ + context/）
Future<List<CreativeItem>> loadMemoryDocs() async {
  if (kIsWeb || !dataSourceAvailable) return _fallbackMemory();
  final items = <CreativeItem>[];
  for (final sub in ['roadmap', 'context']) {
    final dir = Directory('$memoryPath/$sub');
    if (!await dir.exists()) continue;
    final files = await dir
        .list()
        .where((e) => e.path.endsWith('.md'))
        .toList();
    files.sort((a, b) => a.path.compareTo(b.path));
    items.addAll(files.map((f) => CreativeItem(
          name: f.path.split('/').last.replaceAll('.md', ''),
          path: f.path,
          category: sub,
        )));
  }
  return items;
}

List<CreativeItem> _fallbackChapters() => const [
      CreativeItem(name: '1_1_咖啡厅重逢', path: '（内置示例）', category: '改稿'),
      CreativeItem(name: '7_1_酒吧表白', path: '（内置示例）', category: '改稿'),
      CreativeItem(name: '11_x 世界扩张（规划中）', path: '（内置示例）', category: '改稿'),
    ];

List<CreativeItem> _fallbackMemory() => const [
      CreativeItem(name: 'fiction（路线图）', path: '（内置示例）', category: 'roadmap'),
      CreativeItem(name: 'fiction-plot（情节组织）', path: '（内置示例）', category: 'context'),
      CreativeItem(name: 'fiction-adaptation（转化方法论）', path: '（内置示例）', category: 'context'),
    ];
