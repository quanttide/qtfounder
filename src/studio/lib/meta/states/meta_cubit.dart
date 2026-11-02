import 'package:flutter_bloc/flutter_bloc.dart';

import '../repos/contract_repository.dart';
import '../models/contract.dart';

/// 元认知页状态：加载中、读失败、读到若干契约。
sealed class MetaState {
  const MetaState();
}

class MetaLoading extends MetaState {
  const MetaLoading();
}

class MetaError extends MetaState {
  const MetaError(this.message);

  final String message;
}

class MetaReady extends MetaState {
  const MetaReady({required this.modules, required this.selected});

  /// 全部契约，按模块名升序。
  final List<ContractModule> modules;

  /// 当前选中的下标。
  final int selected;

  ContractModule get current => modules[selected];

  MetaReady copyWith({int? selected}) => MetaReady(
    modules: modules,
    selected: selected ?? this.selected,
  );
}

/// 装载全部契约并维护当前选中项。
class MetaCubit extends Cubit<MetaState> {
  MetaCubit(this.repository) : super(const MetaLoading());

  final ContractRepository repository;

  Future<void> load() async {
    try {
      final modules = await repository.load();
      emit(MetaReady(modules: modules, selected: 0));
    } on Exception catch (error) {
      emit(MetaError('$error'));
    }
  }

  void select(int index) {
    final current = state;
    if (current is MetaReady && index >= 0 && index < current.modules.length) {
      emit(current.copyWith(selected: index));
    }
  }
}
