import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../repos/journal_repository.dart';
import '../states/data_cubit.dart';
import 'label_page.dart';
import 'save_page.dart';

/// 数据标注入口：装配 Cubit，按状态分发页面。
///
/// 仓库根默认按 [JournalRepository.resolve] 解析（当前目录向上找
/// quanttide-founder 根），测试可注入仓库。
class DataAnnotatePage extends StatelessWidget {
  const DataAnnotatePage({super.key, this.repository});

  /// 注入用仓库；缺省时解析本地 quanttide-founder 根。
  final JournalRepository? repository;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<DataCubit>(
      create: (_) =>
          DataCubit(repository ?? JournalRepository.resolve())..load(),
      child: const DataShell(),
    );
  }
}

/// 按批次状态分发到各页面。
class DataShell extends StatelessWidget {
  const DataShell({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<DataCubit, DataState>(
      builder: (context, state) => switch (state) {
        DataLoading() => const DataMessagePage.loading(),
        DataEmpty() => const DataMessagePage.empty(),
        DataError(:final message) => DataMessagePage.error(message: message),
        DataReady() => LabelPage(state: state),
        DataSave() => SavePage(state: state),
        DataSaving(:final total, :final done) =>
          DataMessagePage.saving(done: done, total: total),
        DataDone(:final saved, :final count) =>
          DataMessagePage.done(saved: saved, count: count),
      },
    );
  }
}

/// 加载中、空、失败、保存中、完成共用的提示页。
class DataMessagePage extends StatelessWidget {
  const DataMessagePage.loading({super.key})
    : icon = Icons.hourglass_empty,
      title = '正在读取 journal',
      detail = null;

  const DataMessagePage.empty({super.key})
    : icon = Icons.notes_outlined,
      title = '没有可标注的段落',
      detail = 'data/journal/ 下没有日志文件';

  const DataMessagePage.error({super.key, required String message})
    : icon = Icons.error_outline,
      title = '读取 journal 失败',
      detail = message;

  const DataMessagePage.saving({super.key, required int done, required int total})
    : icon = Icons.save_outlined,
      title = '正在保存标注',
      detail = '$done / $total';

  const DataMessagePage.done({super.key, required String saved, required int count})
    : icon = Icons.check_circle_outline,
      title = '已保存 $count 条标注',
      detail = saved;

  final IconData icon;
  final String title;
  final String? detail;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('数据标注')),
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
                onPressed: () => context.read<DataCubit>().load(),
                child: const Text('再抽一批'),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
