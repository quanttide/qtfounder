import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../repos/data_store.dart';
import '../repos/journal_repository.dart';
import '../states/editor_cubit.dart';

/// 试标：三段真样本按草稿标签走一遍，验的是规则不是数据。
///
/// 标完或跳完自动回编辑器，结果不落盘——试标只是设计时的反馈。
class TrialPage extends StatelessWidget {
  const TrialPage({super.key, required this.store, required this.journal});

  final DataStore store;
  final JournalRepository journal;

  @override
  Widget build(BuildContext context) {
    return BlocConsumer<EditorCubit, EditorState>(
      listenWhen: (previous, current) => current.trial == null,
      listener: (context, _) => Navigator.of(context).pop(),
      buildWhen: (previous, current) =>
          previous.trial?.current != current.trial?.current ||
          previous.draft.labels != current.draft.labels,
      builder: (context, state) {
        final trial = state.trial;
        if (trial == null) {
          return const Scaffold(body: SizedBox.shrink());
        }
        if (trial.done) {
          return _TrialSummary(session: trial);
        }
        final segment = trial.segment;
        final draft = state.draft;
        return Scaffold(
          appBar: AppBar(
            title: Text('试标 ${trial.current + 1} / ${trial.segments.length}'),
          ),
          body: Column(
            children: [
              Expanded(
                child: Padding(
                  padding: const EdgeInsets.all(24),
                  child: DecoratedBox(
                    decoration: BoxDecoration(
                      color: Theme.of(context).colorScheme.surfaceContainerHighest,
                      borderRadius: BorderRadius.circular(12),
                    ),
                    child: ClipRRect(
                      borderRadius: BorderRadius.circular(12),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Padding(
                            padding: const EdgeInsets.fromLTRB(16, 12, 16, 12),
                            child: Text(
                              segment.id,
                              overflow: TextOverflow.ellipsis,
                              style: Theme.of(context).textTheme.bodySmall,
                            ),
                          ),
                          const Divider(height: 1),
                          Expanded(
                            child: SingleChildScrollView(
                              padding: const EdgeInsets.all(16),
                              child: SelectableText(
                                segment.text,
                                style: const TextStyle(fontSize: 15, height: 1.7),
                              ),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                ),
              ),
              Padding(
                padding: const EdgeInsets.fromLTRB(24, 0, 24, 12),
                child: Wrap(
                  spacing: 12,
                  runSpacing: 12,
                  alignment: WrapAlignment.center,
                  children: [
                    for (final label in draft.labels)
                      if (label.name.trim().isNotEmpty)
                        FilledButton(
                          onPressed: () => context.read<EditorCubit>().trialLabel(label.name),
                          child: Text(label.label),
                        ),
                  ],
                ),
              ),
              Padding(
                padding: const EdgeInsets.only(bottom: 16),
                child: TextButton(
                  onPressed: () => context.read<EditorCubit>().trialSkip(),
                  child: const Text('跳过'),
                ),
              ),
            ],
          ),
        );
      },
    );
  }
}

class _TrialSummary extends StatelessWidget {
  const _TrialSummary({required this.session});

  final TrialSession session;

  @override
  Widget build(BuildContext context) {
    final labeled = session.labels.values.where((name) => name.isNotEmpty).length;
    final skipped = session.skipped.length;
    return Scaffold(
      appBar: AppBar(title: const Text('试标结果')),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(32),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.check_circle_outline, size: 56, color: Theme.of(context).colorScheme.primary),
              const SizedBox(height: 16),
              Text('标了 $labeled 段，跳过 $skipped 段', style: Theme.of(context).textTheme.titleLarge),
              const SizedBox(height: 8),
              const Text(
                '跳过率高说明标签定义没盖住这段内容——回去改定义或切法再试',
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 24),
              FilledButton(
                onPressed: () => context.read<EditorCubit>().endTrial(),
                child: const Text('回编辑器'),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
