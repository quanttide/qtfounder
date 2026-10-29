import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../models/rule.dart';
import '../repos/data_store.dart';
import '../repos/journal_repository.dart';
import '../states/editor_cubit.dart';
import '../states/rules_cubit.dart';
import 'editor_page.dart';

/// 数据页入口：装配存储与规则列表 Cubit。
///
/// 存储默认按 [DataStore.resolve] 落 XDG（与 CLI `qtfounder data` 同一处），
/// 测试可注入。
class DataRulesPage extends StatelessWidget {
  const DataRulesPage({super.key, this.store, this.journal});

  final DataStore? store;
  final JournalRepository? journal;

  @override
  Widget build(BuildContext context) {
    final resolvedStore = store ?? DataStore.resolve();
    return BlocProvider<RulesCubit>(
      create: (_) => RulesCubit(resolvedStore)..load(),
      child: _RulesView(
        journal: journal ?? JournalRepository.resolve(),
        store: resolvedStore,
      ),
    );
  }
}

class _RulesView extends StatelessWidget {
  const _RulesView({required this.journal, required this.store});

  final JournalRepository journal;
  final DataStore store;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('标注规则')),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: () => _openEditor(context, null),
        icon: const Icon(Icons.add),
        label: const Text('设计规则'),
      ),
      body: BlocBuilder<RulesCubit, RulesState>(
        builder: (context, state) => switch (state) {
          RulesLoading() => const Center(child: CircularProgressIndicator()),
          RulesError(:final message) => Center(
            child: Padding(
              padding: const EdgeInsets.all(32),
              child: SelectableText('读取规则失败：$message'),
            ),
          ),
          RulesList(:final rules, :final taskCount) => rules.isEmpty
              ? const _EmptyRules()
              : _RuleList(
                  rules: rules,
                  taskCount: taskCount,
                  onOpen: (rule) => _openEditor(context, rule),
                ),
        },
      ),
    );
  }

  /// 打开编辑器：给的是新草稿还是已有规则，编辑完回列表刷新。
  Future<void> _openEditor(BuildContext context, Rule? rule) async {
    final rulesCubit = context.read<RulesCubit>();
    final draft = rule ?? rulesCubit.newDraft();
    await Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => BlocProvider<EditorCubit>(
          create: (_) => EditorCubit(store, journal, draft: draft),
          child: EditorPage(store: store, journal: journal),
        ),
      ),
    );
    await rulesCubit.load();
  }
}

class _EmptyRules extends StatelessWidget {
  const _EmptyRules();

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.rule_outlined, size: 56, color: Theme.of(context).colorScheme.outline),
            const SizedBox(height: 16),
            const Text('还没有标注规则', style: TextStyle(fontSize: 18)),
            const SizedBox(height: 8),
            const Text(
              '一条规则 = 标注对象 + 标签集 + 批次；设计好导出任务包给标注端',
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
    );
  }
}

class _RuleList extends StatelessWidget {
  const _RuleList({required this.rules, required this.taskCount, required this.onOpen});

  final List<Rule> rules;
  final int taskCount;
  final void Function(Rule) onOpen;

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 88),
      children: [
        Padding(
          padding: const EdgeInsets.only(bottom: 12, left: 4),
          child: Text(
            '${rules.length} 条规则 · 已导出 $taskCount 个任务包',
            style: Theme.of(context).textTheme.bodySmall,
          ),
        ),
        for (final rule in rules)
          Card(
            margin: const EdgeInsets.only(bottom: 8),
            child: ListTile(
              title: Text(rule.name),
              subtitle: Text(
                '${rule.labels.map((label) => label.label).join('、')} · 批次 ${rule.batch}',
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
              trailing: const Icon(Icons.chevron_right),
              onTap: () => onOpen(rule),
            ),
          ),
      ],
    );
  }
}
