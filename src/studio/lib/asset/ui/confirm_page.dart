import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../bloc/asset_cubit.dart';
import '../model/asset_file.dart';

/// 确认页：列出所有标记删除的资产，最后一次确认。
class ConfirmPage extends StatelessWidget {
  const ConfirmPage({super.key, required this.state});

  final AssetConfirm state;

  @override
  Widget build(BuildContext context) {
    final marked = state.marked;

    return Scaffold(
      appBar: AppBar(title: const Text('确认')),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(24, 16, 24, 8),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  '将删除 ${marked.length} 个资产',
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
                const SizedBox(height: 4),
                Text(
                  '保留 ${state.keptCount} 个。删除的文件移入 .trash/，'
                  '保留目录结构，不直接删除。',
                ),
              ],
            ),
          ),
          Expanded(
            child: marked.isEmpty
                ? const Center(child: Text('没有标记删除的资产。'))
                : ListView.separated(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 24,
                      vertical: 8,
                    ),
                    itemCount: marked.length,
                    separatorBuilder: (_, _) => const Divider(height: 1),
                    itemBuilder: (context, index) {
                      final asset = marked[index];
                      return ListTile(
                        contentPadding: EdgeInsets.zero,
                        leading: Icon(
                          asset.kind == AssetKind.document
                              ? Icons.description_outlined
                              : Icons.code,
                        ),
                        title: Text(asset.id),
                        subtitle: Text(
                          '${asset.sizeLabel} · ${asset.modifiedLabel}',
                        ),
                      );
                    },
                  ),
          ),
          Padding(
            padding: const EdgeInsets.all(24),
            child: SizedBox(
              width: double.infinity,
              child: FilledButton.icon(
                onPressed: marked.isEmpty
                    ? null
                    : () => context.read<AssetCubit>().confirm(),
                icon: const Icon(Icons.delete_sweep_outlined),
                label: const Text('移入回收目录'),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// 提交中：逐个移动的进度。
class SubmittingPage extends StatelessWidget {
  const SubmittingPage({super.key, required this.state});

  final AssetSubmitting state;

  @override
  Widget build(BuildContext context) {
    final total = state.total == 0 ? 1 : state.total;
    return Scaffold(
      appBar: AppBar(title: const Text('正在移动')),
      body: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            SizedBox(
              width: 320,
              child: LinearProgressIndicator(value: state.done / total),
            ),
            const SizedBox(height: 16),
            Text('${state.done} / ${state.total}'),
          ],
        ),
      ),
    );
  }
}
