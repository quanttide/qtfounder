import 'dart:convert';
import 'dart:io';

import 'package:path/path.dart' as p;

import '../models/rule.dart';

/// 任务包里的一个待标单元（journal 切出来的一段）。
class Unit {
  const Unit({required this.id, required this.domain, required this.text});

  final String id;
  final String domain;
  final String text;

  Map<String, dynamic> toJson() => {'id': id, 'domain': domain, 'text': text};
}

/// 任务包：规则快照 + 待标单元，标注端（lite）只认它。
class TaskPackage {
  const TaskPackage({
    required this.id,
    required this.ruleId,
    required this.rule,
    required this.units,
    required this.at,
  });

  final String id;
  final String ruleId;
  final Rule rule;
  final List<Unit> units;
  final String at;

  Map<String, dynamic> toJson() => {
    'id': id,
    'rule_id': ruleId,
    'rule': rule.toJson(),
    'units': [for (final unit in units) unit.toJson()],
    'at': at,
  };
}

/// 规则与任务包的存储，落在 XDG 数据目录（与 CLI `qtfounder data` 同一处）：
/// `<data_home>/data/rules/<id>.json`、`<data_home>/data/tasks/<id>.json`。
///
/// `QTFOUNDER_DATA` 优先，其次 `$XDG_DATA_HOME/qtfounder`，再次
/// `~/.local/share/qtfounder`——与 CLI 的解析顺序一致，两边看到同一份数据。
class DataStore {
  DataStore({required this.root});

  /// 数据根目录（XDG 下的 `qtfounder/`，或 `QTFOUNDER_DATA`）。
  final Directory root;

  Directory get rulesDir => Directory(p.join(root.path, 'data', 'rules'));
  Directory get tasksDir => Directory(p.join(root.path, 'data', 'tasks'));

  /// 解析数据根：`QTFOUNDER_DATA` → `$XDG_DATA_HOME/qtfounder` →
  /// `~/.local/share/qtfounder`。
  factory DataStore.resolve() {
    return DataStore(
      root: Directory(
        rootFrom(
          Platform.environment['QTFOUNDER_DATA'],
          Platform.environment['XDG_DATA_HOME'],
          Platform.environment['HOME'],
        ),
      ),
    );
  }

  /// 纯函数形式的根解析，便于测试钉住分支。
  static String rootFrom(String? overrideDir, String? xdg, String? home) {
    if (overrideDir != null && overrideDir.trim().isNotEmpty) {
      return overrideDir;
    }
    if (xdg != null && xdg.trim().isNotEmpty) {
      return p.join(xdg, 'qtfounder');
    }
    if (home != null && home.trim().isNotEmpty) {
      return p.join(home, '.local', 'share', 'qtfounder');
    }
    return 'qtfounder';
  }

  /// 全部规则，按 id 升序；目录不存在按空。
  Future<List<Rule>> rules() async {
    if (!await rulesDir.exists()) return [];
    final files = await rulesDir
        .list()
        .where((entry) => entry.path.endsWith('.json'))
        .toList();
    final rules = <Rule>[];
    for (final file in files) {
      final text = await File(file.path).readAsString();
      rules.add(Rule.fromJson(jsonDecode(text) as Map<String, dynamic>));
    }
    rules.sort((a, b) => a.id.compareTo(b.id));
    return rules;
  }

  /// 写入一条规则（同 id 覆盖），返回落盘路径。
  Future<String> saveRule(Rule rule) async {
    await rulesDir.create(recursive: true);
    final path = p.join(rulesDir.path, '${rule.id}.json');
    await File(path).writeAsString('${const JsonEncoder.withIndent('  ').convert(rule.toJson())}\n');
    return path;
  }

  /// 导出一个任务包，返回落盘路径。
  Future<String> exportTask(TaskPackage task) async {
    await tasksDir.create(recursive: true);
    final path = p.join(tasksDir.path, '${task.id}.json');
    await File(path).writeAsString('${const JsonEncoder.withIndent('  ').convert(task.toJson())}\n');
    return path;
  }

  /// 已导出的任务包数量（列表页显示用）。
  Future<int> taskCount() async {
    if (!await tasksDir.exists()) return 0;
    return tasksDir
        .list()
        .where((entry) => entry.path.endsWith('.json'))
        .length;
  }
}
