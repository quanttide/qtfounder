/// 资产目录模型——通用目录树（由资产契约驱动）
library;

/// 资产契约（解析自 contracts/*.yaml）
class AssetContract {
  final String asset;
  final String label;
  final String root;
  final List<String> ignore;
  final List<LevelSpec> levels;
  final NamingSpec naming;

  const AssetContract({
    required this.asset,
    required this.label,
    required this.root,
    required this.ignore,
    required this.levels,
    required this.naming,
  });
}

/// 层级语义（levels 的一项）
class LevelSpec {
  final String key;
  final String label;
  final bool optional;

  const LevelSpec({
    required this.key,
    required this.label,
    this.optional = false,
  });
}

/// 文件命名规则（naming）
class NamingSpec {
  final String? pattern; // 正则；null = 自由命名
  final String? title; // 标题组引用（$1/$2/...）；null = 全名
  final List<String> sortKey; // 排序键组引用
  final String? version; // 版本组引用；null = 1
  final String unsorted; // last（已排序在前）/ natural（全部自然序）

  const NamingSpec({
    this.pattern,
    this.title,
    this.sortKey = const [],
    this.version,
    this.unsorted = 'last',
  });
}

/// 目录节点（契约 levels 的每一级）
class CatalogNode {
  final String name; // 目录名
  final String label; // 契约语义（小说/阶段/类型...）
  final List<CatalogNode> children; // 子目录
  final List<CatalogFile> files; // 文件（已排序）

  const CatalogNode({
    required this.name,
    required this.label,
    required this.children,
    required this.files,
  });
}

/// 文件（契约 naming 解析结果）
class CatalogFile {
  final String name; // 文件名（含扩展名）
  final String path; // 绝对路径
  final String title; // 展示名（解析后）
  final String? sortKey; // 排序键（编号 "1_1" / 日期 / null）
  final int version; // 版本（默认 1）

  const CatalogFile({
    required this.name,
    required this.path,
    required this.title,
    this.sortKey,
    this.version = 1,
  });
}

/// 资产目录（契约 + 构建结果）
class AssetCatalog {
  final AssetContract contract;
  final List<CatalogNode> nodes; // 顶层节点

  const AssetCatalog({required this.contract, required this.nodes});
}
