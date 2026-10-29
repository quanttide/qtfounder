import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../models/journal_segment.dart';
import '../states/data_cubit.dart';

/// 段落流：一段 journal 文本一屏，点标签即标注并前进，可跳过。
class LabelPage extends StatelessWidget {
  const LabelPage({super.key, required this.state});

  final DataReady state;

  @override
  Widget build(BuildContext context) {
    final segment = state.segment;

    return Scaffold(
      appBar: AppBar(
        title: Text('还剩 ${state.remaining} 段 · 已标 ${state.labeledCount}'),
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
                        child: Row(
                          children: [
                            _DomainBadge(domain: segment.domain),
                            const SizedBox(width: 8),
                            Expanded(
                              child: Text(
                                segment.id,
                                overflow: TextOverflow.ellipsis,
                                style: Theme.of(context).textTheme.bodySmall,
                              ),
                            ),
                          ],
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
                for (final label in AnnotateLabel.values)
                  FilledButton(
                    onPressed: () => context.read<DataCubit>().label(label),
                    child: Text(label.label),
                  ),
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.only(bottom: 16),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                TextButton(
                  onPressed: () => context.read<DataCubit>().skip(),
                  child: const Text('跳过'),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _DomainBadge extends StatelessWidget {
  const _DomainBadge({required this.domain});

  final String domain;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
      decoration: BoxDecoration(
        color: Colors.indigo.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(6),
      ),
      child: Text(
        domain,
        style: const TextStyle(
          color: Colors.indigo,
          fontSize: 12,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }
}
