import 'dart:io';
import 'dart:math';

import 'package:flutter_bloc/flutter_bloc.dart';

import '../repos/asset_repository.dart';
import '../models/asset_file.dart';

/// 批次状态。卡片流走 AssetReady，栈空进 AssetConfirm，提交进
/// AssetSubmitting，提交完进 AssetDone。
sealed class AssetState {
  const AssetState();
}

class AssetLoading extends AssetState {
  const AssetLoading();
}

class AssetReady extends AssetState {
  const AssetReady({
    required this.assets,
    required this.current,
    required this.decisions,
    required this.previews,
  });

  final List<AssetFile> assets;

  /// 当前卡片下标。
  final int current;

  /// asset.id → 决策。
  final Map<String, Decision> decisions;

  /// asset.id → 文件内容预览。
  final Map<String, String> previews;

  AssetFile get asset => assets[current];

  int get remaining => assets.length - current;

  int get deletedCount =>
      decisions.values.where((decision) => decision == Decision.delete).length;
}

class AssetConfirm extends AssetState {
  const AssetConfirm({required this.assets, required this.decisions});

  final List<AssetFile> assets;
  final Map<String, Decision> decisions;

  List<AssetFile> get marked => [
    for (final asset in assets)
      if (decisions[asset.id] == Decision.delete) asset,
  ];

  int get keptCount => assets.length - marked.length;
}

class AssetSubmitting extends AssetState {
  const AssetSubmitting({required this.total, required this.done});

  final int total;
  final int done;
}

class AssetDone extends AssetState {
  const AssetDone({required this.succeeded, required this.failed});

  final List<AssetFile> succeeded;
  final List<MoveFailure> failed;
}

class AssetEmpty extends AssetState {
  const AssetEmpty({required this.path});

  final String path;
}

class AssetError extends AssetState {
  const AssetError(this.message);

  final String message;
}

/// 一个资产移动失败。
class MoveFailure {
  const MoveFailure({required this.asset, required this.message});

  final AssetFile asset;
  final String message;
}

class AssetCubit extends Cubit<AssetState> {
  AssetCubit(this._repository, {this.batchSize = 20, Random? random})
    : _random = random ?? Random(),
      super(const AssetLoading());

  final AssetRepository _repository;
  final int batchSize;
  final Random _random;

  Future<void> load() async {
    emit(const AssetLoading());
    try {
      final assets = await _repository.scan();
      if (assets.isEmpty) {
        emit(AssetEmpty(path: _repository.root.path));
        return;
      }
      assets.shuffle(_random);
      final batch = assets.take(batchSize).toList(growable: false);

      final previews = <String, String>{};
      for (final asset in batch) {
        try {
          previews[asset.id] = await _repository.preview(asset);
        } catch (_) {
          previews[asset.id] = '（无法读取预览）';
        }
      }

      emit(
        AssetReady(
          assets: batch,
          current: 0,
          decisions: const {},
          previews: previews,
        ),
      );
    } catch (error) {
      emit(AssetError(_describe(error)));
    }
  }

  void decide(Decision decision) {
    final state = this.state;
    if (state is! AssetReady) return;
    final decisions = {...state.decisions, state.asset.id: decision};
    final next = state.current + 1;
    if (next >= state.assets.length) {
      emit(AssetConfirm(assets: state.assets, decisions: decisions));
    } else {
      emit(
        AssetReady(
          assets: state.assets,
          current: next,
          decisions: decisions,
          previews: state.previews,
        ),
      );
    }
  }

  Future<void> confirm() async {
    final state = this.state;
    if (state is! AssetConfirm) return;

    final targets = state.marked;
    final succeeded = <AssetFile>[];
    final failed = <MoveFailure>[];
    emit(AssetSubmitting(total: targets.length, done: 0));

    for (var index = 0; index < targets.length; index++) {
      final asset = targets[index];
      try {
        await _repository.moveToTrash(asset);
        succeeded.add(asset);
      } catch (error) {
        failed.add(MoveFailure(asset: asset, message: _describe(error)));
      }
      emit(AssetSubmitting(total: targets.length, done: index + 1));
    }

    emit(AssetDone(succeeded: succeeded, failed: failed));
  }

  static String _describe(Object error) => switch (error) {
    AssetScanException(:final message) => message,
    FileSystemException(:final message, :final path) =>
      path == null ? message : '$message（$path）',
    _ => error.toString(),
  };
}
