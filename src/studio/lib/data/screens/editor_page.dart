import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../models/rule.dart';
import '../repos/data_store.dart';
import '../repos/journal_repository.dart';
import '../states/editor_cubit.dart';
import 'trial_page.dart';

/// 规则编辑器：规则名、标签集（机器名/显示名/定义/正反例）、批次；
/// 试标验规则，保存落 XDG，导出任务包给标注端。
class EditorPage extends StatelessWidget {
  const EditorPage({super.key, required this.store, required this.journal});

  final DataStore store;
  final JournalRepository journal;

  @override
  Widget build(BuildContext context) {
    String? lastMessage;
    return BlocConsumer<EditorCubit, EditorState>(
      listener: (context, state) {
        if (state.trial != null) {
          Navigator.of(context).push(
            MaterialPageRoute<void>(
              builder: (_) => TrialPage(store: store, journal: journal),
            ),
          );
        }
        if (state.error != null) return;
        final message = switch ((state.savedPath, state.exportedPath)) {
          (final String saved, _) => '已保存规则：$saved',
          (_, final String exported) => '已导出任务包：$exported',
          _ => null,
        };
        if (message != null && message != lastMessage) {
          lastMessage = message;
          ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(message)));
        }
      },
      buildWhen: (previous, current) =>
          previous.draft != current.draft ||
          previous.error != current.error ||
          previous.trial != null,
      builder: (context, state) {
        final cubit = context.read<EditorCubit>();
        final draft = state.draft;
        return Scaffold(
          appBar: AppBar(title: Text(draft.name.isEmpty ? '设计规则' : draft.name)),
          body: ListView(
            padding: const EdgeInsets.all(16),
            children: [
              if (state.error != null) ...[
                _ErrorBanner(message: state.error!),
                const SizedBox(height: 12),
              ],
              Text('规则名', style: Theme.of(context).textTheme.titleSmall),
              TextField(
                decoration: const InputDecoration(hintText: '如：日志四分类'),
                controller: TextEditingController(text: draft.name)
                  ..selection = TextSelection.collapsed(offset: draft.name.length),
                onChanged: cubit.rename,
              ),
              const SizedBox(height: 16),
              Text('标签集', style: Theme.of(context).textTheme.titleSmall),
              const SizedBox(height: 4),
              Text(
                '机器名进结果文件，显示名上界面；定义与正反例把判断标准说清楚',
                style: Theme.of(context).textTheme.bodySmall,
              ),
              const SizedBox(height: 8),
              for (var index = 0; index < draft.labels.length; index++) ...[
                _LabelCard(
                  label: draft.labels[index],
                  onChanged: (label) => cubit.updateLabel(index, label),
                  onRemove: draft.labels.length > 1 ? () => cubit.removeLabel(index) : null,
                ),
                const SizedBox(height: 8),
              ],
              TextButton.icon(
                onPressed: cubit.addLabel,
                icon: const Icon(Icons.add),
                label: const Text('加一个标签'),
              ),
              const SizedBox(height: 16),
              Text('批次', style: Theme.of(context).textTheme.titleSmall),
              Row(
                children: [
                  SizedBox(
                    width: 96,
                    child: TextField(
                      keyboardType: TextInputType.number,
                      controller: TextEditingController(text: '${draft.batch}')
                        ..selection = TextSelection.collapsed(offset: '${draft.batch}'.length),
                      onChanged: (text) => cubit.setBatch(int.tryParse(text) ?? 0),
                    ),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      '任务包一次给标注端多少段',
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 24),
              FilledButton.icon(
                onPressed: cubit.startTrial,
                icon: const Icon(Icons.fact_check_outlined),
                label: const Text('试标 3 段：验规则'),
              ),
              const SizedBox(height: 8),
              FilledButton.tonalIcon(
                onPressed: cubit.save,
                icon: const Icon(Icons.save_outlined),
                label: const Text('保存规则'),
              ),
              const SizedBox(height: 8),
              OutlinedButton.icon(
                onPressed: cubit.exportTask,
                icon: const Icon(Icons.ios_share),
                label: const Text('导出任务包'),
              ),
              if (state.exportedPath != null) ...[
                const SizedBox(height: 8),
                SelectableText(
                  '已导出：${state.exportedPath}',
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ],
              const SizedBox(height: 32),
            ],
          ),
        );
      },
    );
  }
}

class _ErrorBanner extends StatelessWidget {
  const _ErrorBanner({required this.message});

  final String message;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: scheme.errorContainer,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Icon(Icons.error_outline, color: scheme.onErrorContainer),
          const SizedBox(width: 8),
          Expanded(child: Text(message, style: TextStyle(color: scheme.onErrorContainer))),
        ],
      ),
    );
  }
}

class _LabelCard extends StatelessWidget {
  const _LabelCard({required this.label, required this.onChanged, this.onRemove});

  final RuleLabel label;
  final ValueChanged<RuleLabel> onChanged;
  final VoidCallback? onRemove;

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: EdgeInsets.zero,
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(
                  child: TextFormField(
                    decoration: const InputDecoration(
                      labelText: '机器名',
                      hintText: 'fact',
                      isDense: true,
                    ),
                    initialValue: label.name,
                    onChanged: (text) => onChanged(label.copyWith(name: text)),
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: TextFormField(
                    decoration: const InputDecoration(
                      labelText: '显示名',
                      hintText: '事实',
                      isDense: true,
                    ),
                    initialValue: label.label,
                    onChanged: (text) => onChanged(label.copyWith(label: text)),
                  ),
                ),
                IconButton(
                  onPressed: onRemove,
                  icon: const Icon(Icons.delete_outline),
                  tooltip: '删掉这个标签',
                ),
              ],
            ),
            TextFormField(
              decoration: const InputDecoration(
                labelText: '定义：什么算这个标签',
                isDense: true,
              ),
              initialValue: label.define,
              onChanged: (text) => onChanged(label.copyWith(define: text)),
            ),
            Row(
              children: [
                Expanded(
                  child: TextFormField(
                    decoration: const InputDecoration(
                      labelText: '正例一句',
                      isDense: true,
                    ),
                    initialValue: label.yes,
                    onChanged: (text) => onChanged(label.copyWith(yes: text)),
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: TextFormField(
                    decoration: const InputDecoration(
                      labelText: '反例一句',
                      isDense: true,
                    ),
                    initialValue: label.no,
                    onChanged: (text) => onChanged(label.copyWith(no: text)),
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
