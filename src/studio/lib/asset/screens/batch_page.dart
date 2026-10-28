import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../states/asset_cubit.dart';
import '../models/asset_file.dart';

/// 卡片流：只渲染栈顶一个资产，左右滑动或点按钮决策。
class BatchPage extends StatelessWidget {
  const BatchPage({super.key, required this.state});

  final AssetReady state;

  @override
  Widget build(BuildContext context) {
    final asset = state.asset;

    return Scaffold(
      appBar: AppBar(
        title: Text('还剩 ${state.remaining} 个 · 已标记删除 ${state.deletedCount}'),
      ),
      body: Column(
        children: [
          Expanded(
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Dismissible(
                key: ValueKey(asset.id),
                direction: DismissDirection.horizontal,
                background: const _SwipeHint(
                  alignment: Alignment.centerLeft,
                  icon: Icons.undo,
                  label: '保留',
                  color: Colors.green,
                ),
                secondaryBackground: const _SwipeHint(
                  alignment: Alignment.centerRight,
                  icon: Icons.delete_outline,
                  label: '删除',
                  color: Colors.red,
                ),
                onDismissed: (direction) {
                  context.read<AssetCubit>().decide(
                    direction == DismissDirection.startToEnd
                        ? Decision.keep
                        : Decision.delete,
                  );
                },
                child: _AssetCard(
                  asset: asset,
                  preview: state.previews[asset.id] ?? '',
                ),
              ),
            ),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(24, 0, 24, 24),
            child: Row(
              children: [
                Expanded(
                  child: FilledButton.icon(
                    onPressed: () =>
                        context.read<AssetCubit>().decide(Decision.delete),
                    icon: const Icon(Icons.delete_outline),
                    label: const Text('删除'),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: FilledButton.tonalIcon(
                    onPressed: () =>
                        context.read<AssetCubit>().decide(Decision.keep),
                    icon: const Icon(Icons.check),
                    label: const Text('保留'),
                  ),
                ),
              ],
            ),
          ),
          const Padding(
            padding: EdgeInsets.only(bottom: 16),
            child: Text('左滑删除 · 右滑保留'),
          ),
        ],
      ),
    );
  }
}

class _AssetCard extends StatelessWidget {
  const _AssetCard({required this.asset, required this.preview});

  final AssetFile asset;
  final String preview;

  @override
  Widget build(BuildContext context) {
    final scheme = Theme.of(context).colorScheme;
    final kindColor = switch (asset.kind) {
      AssetKind.document => Colors.indigo,
      AssetKind.code => Colors.teal,
    };

    return DecoratedBox(
      decoration: BoxDecoration(
        color: scheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(12),
      ),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 12, 16, 12),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      _KindBadge(label: asset.kindLabel, color: kindColor),
                      const SizedBox(width: 8),
                      Text(
                        '${asset.sizeLabel} · ${asset.modifiedLabel}',
                        style: Theme.of(context).textTheme.bodySmall,
                      ),
                    ],
                  ),
                  const SizedBox(height: 8),
                  SelectableText(
                    asset.id,
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                ],
              ),
            ),
            const Divider(height: 1),
            Expanded(
              child: SingleChildScrollView(
                padding: const EdgeInsets.all(16),
                child: SelectableText(
                  preview,
                  style: const TextStyle(
                    fontFamily: 'monospace',
                    fontSize: 13,
                    height: 1.5,
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _KindBadge extends StatelessWidget {
  const _KindBadge({required this.label, required this.color});

  final String label;
  final Color color;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(6),
      ),
      child: Text(
        label,
        style: TextStyle(
          color: color,
          fontSize: 12,
          fontWeight: FontWeight.w600,
        ),
      ),
    );
  }
}

class _SwipeHint extends StatelessWidget {
  const _SwipeHint({
    required this.alignment,
    required this.icon,
    required this.label,
    required this.color,
  });

  final Alignment alignment;
  final IconData icon;
  final String label;
  final Color color;

  @override
  Widget build(BuildContext context) {
    return Container(
      alignment: alignment,
      padding: const EdgeInsets.symmetric(horizontal: 32),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.15),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, color: color, size: 48),
          Text(label, style: TextStyle(color: color, fontSize: 18)),
        ],
      ),
    );
  }
}

/// 加载中、空仓库、读取失败共用的提示页。
class BatchMessagePage extends StatelessWidget {
  const BatchMessagePage.loading({super.key})
    : icon = Icons.hourglass_empty,
      title = '正在扫描仓库',
      detail = null;

  const BatchMessagePage.empty({super.key, required String path})
    : icon = Icons.folder_open_outlined,
      title = '没有可抽查的资产',
      detail = path;

  const BatchMessagePage.error({super.key, required String message})
    : icon = Icons.error_outline,
      title = '扫描仓库失败',
      detail = message;

  final IconData icon;
  final String title;
  final String? detail;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('资产抽查')),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(32),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(icon, size: 56),
              const SizedBox(height: 16),
              Text(title, style: Theme.of(context).textTheme.titleLarge),
              if (detail != null) ...[
                const SizedBox(height: 8),
                SelectableText(detail!, textAlign: TextAlign.center),
              ],
              const SizedBox(height: 24),
              FilledButton(
                onPressed: () => context.read<AssetCubit>().load(),
                child: const Text('重新扫描'),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
