import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../bloc/asset_cubit.dart';

/// 结果页：成功与失败的清单，失败的逐条列出。
class ResultPage extends StatelessWidget {
  const ResultPage({super.key, required this.state});

  final AssetDone state;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('结果')),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(24, 16, 24, 8),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  '已移入回收目录 ${state.succeeded.length} 个',
                  style: Theme.of(context).textTheme.headlineSmall,
                ),
                if (state.failed.isNotEmpty)
                  Text(
                    '失败 ${state.failed.length} 个',
                    style: TextStyle(
                      color: Theme.of(context).colorScheme.error,
                    ),
                  ),
              ],
            ),
          ),
          Expanded(
            child: state.failed.isEmpty
                ? const Center(child: Text('全部完成。'))
                : ListView.separated(
                    padding: const EdgeInsets.all(24),
                    itemCount: state.failed.length,
                    separatorBuilder: (_, _) => const Divider(height: 1),
                    itemBuilder: (context, index) {
                      final failure = state.failed[index];
                      return ListTile(
                        contentPadding: EdgeInsets.zero,
                        leading: const Icon(Icons.error_outline),
                        title: Text(failure.asset.id),
                        subtitle: Text(failure.message),
                      );
                    },
                  ),
          ),
          Padding(
            padding: const EdgeInsets.all(24),
            child: SizedBox(
              width: double.infinity,
              child: FilledButton.icon(
                onPressed: () => context.read<AssetCubit>().load(),
                icon: const Icon(Icons.refresh),
                label: const Text('再来一批'),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
