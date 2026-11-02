/// 元认知模块读到的一个契约条目：键、值，以及它上方的注释。
class ContractEntry {
  const ContractEntry({
    required this.key,
    required this.value,
    this.comments = const [],
  });

  final String key;

  /// 渲染后的值（字符串原样，数组与表压成一行）。
  final String value;

  /// 紧挨这条键上方的注释行。
  final List<String> comments;
}

/// 契约里的一段（TOML 表或数组表）：段名、条目与表头上的注释。
class ContractSection {
  const ContractSection({
    required this.name,
    this.comments = const [],
    this.entries = const [],
  });

  /// 段名，如 `scope`、`banned.不变式`、`asset[0]`、`(顶层)`。
  final String name;

  /// 紧挨这个表头上方的注释行。
  final List<String> comments;

  final List<ContractEntry> entries;
}

/// 一个 CLI 模块的契约：结构（段与条目）、原文，以及它引用到的其他模块。
class ContractModule {
  const ContractModule({
    required this.name,
    required this.domain,
    required this.path,
    required this.source,
    required this.sections,
    this.references = const [],
  });

  /// 模块名（契约所在目录名），如 `language`。
  final String name;

  /// 契约里的 `domain` 字段，缺省回退为模块名。
  final String domain;

  /// 契约文件的绝对路径。
  final String path;

  /// 契约原文。
  final String source;

  /// 结构化的段。
  final List<ContractSection> sections;

  /// 本契约正文里提到的其他模块名（升序）。
  final List<String> references;

  ContractModule withReferences(List<String> references) => ContractModule(
    name: name,
    domain: domain,
    path: path,
    source: source,
    sections: sections,
    references: references,
  );

  /// 条目总数。
  int get entryCount =>
      sections.fold(0, (sum, section) => sum + section.entries.length);
}
