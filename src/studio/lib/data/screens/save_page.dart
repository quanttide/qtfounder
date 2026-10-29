import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../states/data_cubit.dart';

/// 保存页：过一遍本批标注（段落与所贴标签），确认后落盘。
class SavePage extends StatelessWidget {
  const SavePage({super.key, required this.state});

  final DataSave state;

  @override
  Widget build(BuildContext context) {
    final annotations = state.annotations.values.toList(growable: false);

    return Scaffold(
      appBar: AppBar(title: Text('本批标注 ${annotations.length} 段')),
      body: Column(
        children: [
          Expanded(
            child: ListView.separated(
              padding: const EdgeInsets.all(16),
              itemCount: annotations.length,
              separatorBuilder: (_, _) => const SizedBox(height: 8),
              itemBuilder: (context, index) {
                final annotation = annotations[index];
                return Card(
                  margin: EdgeInsets.zero,
                  child: Padding(
                    padding: const EdgeInsets.all(12),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          children: [
                            Chip(
                              label: Text(annotation.label.label),
                              visualDensity: VisualDensity.compact,
                            ),
                            const SizedBox(width: 8),
                            Expanded(
                              child: Text(
                                annotation.segment.id,
                                overflow: TextOverflow.ellipsis,
                                style: Theme.of(context).textTheme.bodySmall,
                              ),
                            ),
                          ],
                        ),
                        const SizedBox(height: 8),
                        Text(
                          annotation.segment.text,
                          maxLines: 3,
                          overflow: TextOverflow.ellipsis,
                          style: const TextStyle(fontSize: 13, height: 1.5),
                        ),
                      ],
                    ),
                  ),
                );
              },
            ),
          ),
          Padding(
            padding: const EdgeInsets.all(24),
            child: Row(
              children: [
                Expanded(
                  child: FilledButton.icon(
                    onPressed: () => context.read<DataCubit>().save(),
                    icon: const Icon(Icons.save_outlined),
                    label: const Text('保存标注'),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
