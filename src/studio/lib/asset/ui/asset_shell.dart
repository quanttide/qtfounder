import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../bloc/asset_cubit.dart';
import '../data/asset_repository.dart';
import 'batch_page.dart';
import 'confirm_page.dart';
import 'result_page.dart';

/// 资产抽查入口：装配 Cubit，按状态分发页面。
///
/// 仓库根默认按 [AssetRepository.resolve] 解析（当前目录向上找
/// quanttide-founder 根），测试可注入仓库。
class AssetAuditPage extends StatelessWidget {
  const AssetAuditPage({super.key, this.repository});

  /// 注入用仓库；缺省时解析本地 quanttide-founder 根。
  final AssetRepository? repository;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<AssetCubit>(
      create: (_) =>
          AssetCubit(repository ?? AssetRepository.resolve())..load(),
      child: const AssetShell(),
    );
  }
}

/// 按批次状态分发到各页面。
class AssetShell extends StatelessWidget {
  const AssetShell({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<AssetCubit, AssetState>(
      builder: (context, state) => switch (state) {
        AssetLoading() => const BatchMessagePage.loading(),
        AssetEmpty(:final path) => BatchMessagePage.empty(path: path),
        AssetError(:final message) => BatchMessagePage.error(message: message),
        AssetReady() => BatchPage(state: state),
        AssetConfirm() => ConfirmPage(state: state),
        AssetSubmitting() => SubmittingPage(state: state),
        AssetDone() => ResultPage(state: state),
      },
    );
  }
}
