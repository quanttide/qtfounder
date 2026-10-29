import 'dart:math';

import 'package:flutter_bloc/flutter_bloc.dart';

import '../models/rule.dart';
import '../repos/data_store.dart';

/// 规则列表状态。
sealed class RulesState {
  const RulesState();
}

class RulesLoading extends RulesState {
  const RulesLoading();
}

class RulesList extends RulesState {
  const RulesList({required this.rules, required this.taskCount});

  final List<Rule> rules;
  final int taskCount;
}

class RulesError extends RulesState {
  const RulesError(this.message);

  final String message;
}

/// 规则列表：读 XDG 存储，管新建后的增改。
class RulesCubit extends Cubit<RulesState> {
  RulesCubit(this._store) : super(const RulesLoading());

  final DataStore _store;

  Future<void> load() async {
    emit(const RulesLoading());
    try {
      final rules = await _store.rules();
      final taskCount = await _store.taskCount();
      emit(RulesList(rules: rules, taskCount: taskCount));
    } catch (error) {
      emit(RulesError(error.toString()));
    }
  }

  /// 新建规则的草稿：id 现生成，标签给一格空位起步。
  Rule newDraft() => Rule(
    id: newRuleId(),
    name: '',
    source: const RuleSource(),
    labels: const [RuleLabel(name: '', label: '')],
    batch: 20,
    at: DateTime.now().toIso8601String(),
  );
}

/// 生成 `rul_XXXX` 形状的 id（Crockford Base32 四位）。
String newRuleId({Random? random}) {
  const alphabet = '0123456789ABCDEFGHJKMNPQRSTVWXYZ';
  final rng = random ?? Random();
  return 'rul_${List.generate(4, (_) => alphabet[rng.nextInt(32)]).join()}';
}
