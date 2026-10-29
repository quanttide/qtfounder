import 'dart:math';

import 'package:flutter_bloc/flutter_bloc.dart';

import '../models/journal_segment.dart';
import '../models/rule.dart';
import '../repos/data_store.dart';
import '../repos/journal_repository.dart';

/// 编辑器状态：草稿 + 校验错误 + 试标与导出的回执。
class EditorState {
  const EditorState({
    required this.draft,
    this.error,
    this.savedPath,
    this.exportedPath,
    this.trial,
  });

  /// 当前草稿。
  final Rule draft;

  /// 最近一次校验或 IO 错误。
  final String? error;

  /// 最近一次保存的规则路径。
  final String? savedPath;

  /// 最近一次导出的任务包路径。
  final String? exportedPath;

  /// 试标会话；非空表示正在试标。
  final TrialSession? trial;

  EditorState copyWith({
    Rule? draft,
    String? error,
    String? savedPath,
    String? exportedPath,
    TrialSession? trial,
    bool clearError = false,
    bool clearTrial = false,
  }) => EditorState(
    draft: draft ?? this.draft,
    error: clearError ? null : (error ?? this.error),
    savedPath: savedPath ?? this.savedPath,
    exportedPath: exportedPath ?? this.exportedPath,
    trial: clearTrial ? null : (trial ?? this.trial),
  );
}

/// 一次试标：三段真样本，按草稿里的标签走一遍，验规则本身。
class TrialSession {
  const TrialSession({
    required this.segments,
    required this.labels,
    required this.skipped,
  });

  final List<JournalSegment> segments;

  /// segment.id → 标签机器名。
  final Map<String, String> labels;

  /// 跳过的 segment.id。
  final Set<String> skipped;

  int get current => labels.length + skipped.length;

  bool get done => current >= segments.length;

  JournalSegment get segment => segments[current];
}

/// 规则编辑器：改草稿、试标、保存规则、导出任务包。
class EditorCubit extends Cubit<EditorState> {
  EditorCubit(
    this._store,
    this._journal, {
    required Rule draft,
    Random? random,
  }) : _random = random ?? Random(),
       super(EditorState(draft: draft));

  final DataStore _store;
  final JournalRepository _journal;
  final Random _random;

  void rename(String name) =>
      emit(state.copyWith(draft: state.draft.copyWith(name: name)));

  void setBatch(int batch) =>
      emit(state.copyWith(draft: state.draft.copyWith(batch: batch)));

  void updateLabel(int index, RuleLabel label) {
    final labels = [...state.draft.labels];
    labels[index] = label;
    emit(state.copyWith(draft: state.draft.copyWith(labels: labels)));
  }

  void addLabel() {
    final labels = [
      ...state.draft.labels,
      const RuleLabel(name: '', label: ''),
    ];
    emit(state.copyWith(draft: state.draft.copyWith(labels: labels)));
  }

  void removeLabel(int index) {
    final labels = [...state.draft.labels]..removeAt(index);
    emit(state.copyWith(draft: state.draft.copyWith(labels: labels)));
  }

  /// 试标：从 journal 抽三段，标签取草稿里机器名非空的。
  Future<void> startTrial() async {
    final error = state.draft.invalid;
    if (error != null) {
      emit(state.copyWith(error: error));
      return;
    }
    try {
      final segments = await _journal.segments();
      if (segments.isEmpty) {
        emit(state.copyWith(error: 'journal 里没有可标注的段落'));
        return;
      }
      final shuffled = [...segments]..shuffle(_random);
      final batch = shuffled.take(3).toList(growable: false);
      emit(
        state.copyWith(
          clearError: true,
          trial: TrialSession(
            segments: batch,
            labels: const {},
            skipped: const {},
          ),
        ),
      );
    } catch (err) {
      emit(state.copyWith(error: err.toString()));
    }
  }

  /// 给试标当前段贴一个标签。
  void trialLabel(String labelName) {
    final trial = state.trial;
    if (trial == null || trial.done) return;
    final labels = {...trial.labels, trial.segment.id: labelName};
    emit(
      state.copyWith(
        trial: TrialSession(
          segments: trial.segments,
          labels: labels,
          skipped: trial.skipped,
        ),
      ),
    );
  }

  /// 跳过试标当前段。
  void trialSkip() {
    final trial = state.trial;
    if (trial == null || trial.done) return;
    final skipped = {...trial.skipped, trial.segment.id};
    emit(
      state.copyWith(
        trial: TrialSession(
          segments: trial.segments,
          labels: trial.labels,
          skipped: skipped,
        ),
      ),
    );
  }

  /// 结束试标，回编辑。
  void endTrial() => emit(state.copyWith(clearTrial: true));

  /// 保存规则到 XDG 存储。
  Future<void> save() async {
    final error = state.draft.invalid;
    if (error != null) {
      emit(state.copyWith(error: error));
      return;
    }
    try {
      final draft = state.draft.copyWith();
      final rule = Rule(
        id: draft.id,
        name: draft.name.trim(),
        source: draft.source,
        labels: draft.labels,
        batch: draft.batch,
        at: DateTime.now().toIso8601String(),
      );
      final path = await _store.saveRule(rule);
      emit(state.copyWith(draft: rule, savedPath: path, clearError: true));
    } catch (err) {
      emit(state.copyWith(error: err.toString()));
    }
  }

  /// 导出任务包：按草稿从 journal 抽 batch 段，规则做快照。
  Future<void> exportTask() async {
    final error = state.draft.invalid;
    if (error != null) {
      emit(state.copyWith(error: error));
      return;
    }
    try {
      final segments = await _journal.segments();
      if (segments.isEmpty) {
        emit(state.copyWith(error: 'journal 里没有可导出的段落'));
        return;
      }
      final shuffled = [...segments]..shuffle(_random);
      final units = shuffled
          .take(state.draft.batch)
          .map((segment) => Unit(id: segment.id, domain: segment.domain, text: segment.text))
          .toList(growable: false);
      final task = TaskPackage(
        id: newTaskId(_random),
        ruleId: state.draft.id,
        rule: state.draft,
        units: units,
        at: DateTime.now().toIso8601String(),
      );
      final path = await _store.exportTask(task);
      emit(state.copyWith(exportedPath: path, clearError: true));
    } catch (err) {
      emit(state.copyWith(error: err.toString()));
    }
  }
}

/// 生成 `tsk_XXXX` 形状的任务 id。
String newTaskId(Random random) {
  const alphabet = '0123456789ABCDEFGHJKMNPQRSTVWXYZ';
  return 'tsk_${List.generate(4, (_) => alphabet[random.nextInt(32)]).join()}';
}
