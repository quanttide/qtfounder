/// 一个待决策的资产文件。
class AssetFile {
  const AssetFile({
    required this.id,
    required this.path,
    required this.kind,
    required this.size,
    required this.modified,
  });

  /// 相对仓库根的路径，批次内唯一。
  final String id;

  /// 绝对路径。
  final String path;

  /// 文档还是代码。
  final AssetKind kind;

  /// 字节数。
  final int size;

  /// 最后修改时间。
  final DateTime modified;

  /// 人类可读的大小。
  String get sizeLabel {
    if (size < 1024) return '$size B';
    if (size < 1024 * 1024) return '${(size / 1024).toStringAsFixed(1)} KB';
    return '${(size / (1024 * 1024)).toStringAsFixed(1)} MB';
  }

  /// 人类可读的修改时间。
  String get modifiedLabel {
    String two(int value) => value.toString().padLeft(2, '0');
    return '${modified.year}-${two(modified.month)}-${two(modified.day)}'
        ' ${two(modified.hour)}:${two(modified.minute)}';
  }

  /// 类型的中文名。
  String get kindLabel => switch (kind) {
    AssetKind.document => '文档',
    AssetKind.code => '代码',
  };
}

/// 资产类型：抽查范围是仓库里的文档与代码。
enum AssetKind { document, code }

/// 用户对一个资产的决策。
enum Decision { keep, delete }
