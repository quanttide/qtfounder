import 'dart:convert';

/// 一个标签：机器名 + 显示名 + 定义与正反例。
///
/// 机器名进结果文件参与统计，显示名上界面；定义与正反例是设计时把
/// 判断标准说清楚的三格，试标时对照用。
class RuleLabel {
  const RuleLabel({
    required this.name,
    required this.label,
    this.define,
    this.yes,
    this.no,
  });

  final String name;
  final String label;
  final String? define;
  final String? yes;
  final String? no;

  RuleLabel copyWith({
    String? name,
    String? label,
    String? define,
    String? yes,
    String? no,
  }) => RuleLabel(
    name: name ?? this.name,
    label: label ?? this.label,
    define: define ?? this.define,
    yes: yes ?? this.yes,
    no: no ?? this.no,
  );

  factory RuleLabel.fromJson(Map<String, dynamic> json) => RuleLabel(
    name: json['name'] as String,
    label: json['label'] as String,
    define: json['define'] as String?,
    yes: json['yes'] as String?,
    no: json['no'] as String?,
  );

  Map<String, dynamic> toJson() => {
    'name': name,
    'label': label,
    if (define != null && define!.trim().isNotEmpty) 'define': define,
    if (yes != null && yes!.trim().isNotEmpty) 'yes': yes,
    if (no != null && no!.trim().isNotEmpty) 'no': no,
  };
}

/// 标注对象：哪来的数据、怎么切段。当前只有 journal / blank-line 一档。
class RuleSource {
  const RuleSource({this.kind = 'journal', this.split = 'blank-line'});

  final String kind;
  final String split;

  factory RuleSource.fromJson(Map<String, dynamic> json) => RuleSource(
    kind: json['kind'] as String? ?? 'journal',
    split: json['split'] as String? ?? 'blank-line',
  );

  Map<String, dynamic> toJson() => {'kind': kind, 'split': split};
}

/// 一条标注规则，字段与 CLI `data rule` 的 JSON 一致。
class Rule {
  const Rule({
    required this.id,
    required this.name,
    required this.source,
    required this.labels,
    required this.batch,
    required this.at,
  });

  final String id;
  final String name;
  final RuleSource source;
  final List<RuleLabel> labels;
  final int batch;
  final String at;

  Rule copyWith({
    String? name,
    RuleSource? source,
    List<RuleLabel>? labels,
    int? batch,
  }) => Rule(
    id: id,
    name: name ?? this.name,
    source: source ?? this.source,
    labels: labels ?? this.labels,
    batch: batch ?? this.batch,
    at: at,
  );

  /// 校验：与 CLI `validate_rule` 同一把尺子，返回第一条不合格的原因。
  String? get invalid {
    if (name.trim().isEmpty) return '规则名不能为空';
    if (batch < 1) return '批次至少 1';
    if (labels.isEmpty) return '至少要有一个标签';
    final seen = <String>{};
    for (final label in labels) {
      if (label.name.trim().isEmpty || label.label.trim().isEmpty) {
        return '标签的机器名与显示名都不能为空';
      }
      if (!seen.add(label.name)) return '标签机器名重复：${label.name}';
    }
    return null;
  }

  factory Rule.fromJson(Map<String, dynamic> json) => Rule(
    id: json['id'] as String,
    name: json['name'] as String,
    source: RuleSource.fromJson(json['source'] as Map<String, dynamic>),
    labels: [
      for (final label in json['labels'] as List)
        RuleLabel.fromJson(label as Map<String, dynamic>),
    ],
    batch: json['batch'] as int? ?? 20,
    at: json['at'] as String? ?? '',
  );

  Map<String, dynamic> toJson() => {
    'id': id,
    'name': name,
    'source': source.toJson(),
    'labels': [for (final label in labels) label.toJson()],
    'batch': batch,
    'at': at,
  };

  String encode() => jsonEncode(toJson());
}
