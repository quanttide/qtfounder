/// 数据源配置——通过环境变量指定 fiction/memory 仓库路径
///
/// 配置方法（dart-define 注入，编译期常量，跨平台一致）：
/// ```bash
/// flutter run --dart-define=QTFOUNDER_FICTION_PATH=/path/to/assets/fiction \
///             --dart-define=QTFOUNDER_MEMORY_PATH=/path/to/assets/memory
/// ```
/// 未配置时回退默认路径（桌面端：~/repos/quanttide-founder/assets/{fiction,memory}）。
library;

import 'dart:io' show Platform;

import 'package:flutter/foundation.dart' show kIsWeb;

/// fiction 数据源路径（环境变量 QTFOUNDER_FICTION_PATH）
String get fictionPath {
  const fromEnv = String.fromEnvironment('QTFOUNDER_FICTION_PATH');
  if (fromEnv.isNotEmpty) return fromEnv;
  return _defaultRepoPath('fiction');
}

/// memory 数据源路径（环境变量 QTFOUNDER_MEMORY_PATH）
String get memoryPath {
  const fromEnv = String.fromEnvironment('QTFOUNDER_MEMORY_PATH');
  if (fromEnv.isNotEmpty) return fromEnv;
  return _defaultRepoPath('memory');
}

/// 桌面端默认路径：$HOME/repos/quanttide-founder/assets/{name}
/// Web 端无文件系统，返回空（由调用方回退内置数据）
String _defaultRepoPath(String asset) {
  if (kIsWeb) return '';
  final home = Platform.environment['HOME'] ?? '';
  if (home.isEmpty) return '';
  return '$home/repos/quanttide-founder/assets/$asset';
}

/// 数据源是否可用（Web 端无文件系统，不可用）
bool get dataSourceAvailable => fictionPath.isNotEmpty && memoryPath.isNotEmpty;
