import 'dart:math';

import 'package:flutter_bloc/flutter_bloc.dart';

import '../models/journal_segment.dart';
import '../repos/journal_repository.dart';

/// 标注批次状态。段落流走 DataReady，栈空进 DataSave，保存中进
/// DataSaving，保存完进 DataDone。
sealed class DataState {
  const DataState();
}

class DataLoading extends DataState {
  const DataLoading();
}

class DataReady extends DataState {
  const DataReady({
    required this.segments,
    required this.current,
    required this.annotations,
  });

  final List<JournalSegment> segments;

  /// 当前段落下标。
  final int current;

  /// segment.id → 标注。
  final Map<String, Annotation> annotations;

  JournalSegment get segment => segments[current];

  int get remaining => segments.length - current;

  int get labeledCount => annotations.length;
}

class DataSave extends DataState {
  const DataSave({required this.annotations});

  final Map<String, Annotation> annotations;
}

class DataSaving extends DataState {
  const DataSaving({required this.total, required this.done});

  final int total;
  final int done;
}

class DataDone extends DataState {
  const DataDone({required this.saved, required this.count});

  /// 写入的文件路径。
  final String saved;

  /// 本次标注条数。
  final int count;
}

class DataEmpty extends DataState {
  const DataEmpty();
}

class DataError extends DataState {
  const DataError(this.message);

  final String message;
}

/// 标注批次：抽一段、贴一个标签、下一段；栈空后一次性落盘。
class DataCubit extends Cubit<DataState> {
  DataCubit(this._repository, {this.batchSize = 20, Random? random})
    : _random = random ?? Random(),
      super(const DataLoading());

  final JournalRepository _repository;
  final int batchSize;
  final Random _random;

  Future<void> load() async {
    emit(const DataLoading());
    try {
      final segments = await _repository.segments();
      if (segments.isEmpty) {
        emit(const DataEmpty());
        return;
      }
      segments.shuffle(_random);
      final batch = segments.take(batchSize).toList(growable: false);
      emit(
        DataReady(segments: batch, current: 0, annotations: const {}),
      );
    } catch (error) {
      emit(DataError(_describe(error)));
    }
  }

  /// 给当前段落贴一个标签，自动前进；贴完最后一段进保存页。
  void label(AnnotateLabel label) {
    final state = this.state;
    if (state is! DataReady) return;
    final annotations = {
      ...state.annotations,
      state.segment.id: Annotation(
        segment: state.segment,
        label: label,
        at: DateTime.now(),
      ),
    };
    final next = state.current + 1;
    if (next >= state.segments.length) {
      emit(DataSave(annotations: annotations));
    } else {
      emit(
        DataReady(
          segments: state.segments,
          current: next,
          annotations: annotations,
        ),
      );
    }
  }

  /// 跳过当前段落，不标注直接前进。
  void skip() {
    final state = this.state;
    if (state is! DataReady) return;
    final next = state.current + 1;
    if (next >= state.segments.length) {
      emit(DataSave(annotations: state.annotations));
    } else {
      emit(
        DataReady(
          segments: state.segments,
          current: next,
          annotations: state.annotations,
        ),
      );
    }
  }

  /// 落盘：把整批标注追加进 JSONL。
  Future<void> save() async {
    final state = this.state;
    if (state is! DataSave) return;
    final list = state.annotations.values.toList(growable: false);
    if (list.isEmpty) {
      // 一条都没标，直接回加载页重新抽。
      await load();
      return;
    }
    emit(DataSaving(total: list.length, done: 0));
    try {
      final saved = await _repository.save(list);
      emit(DataDone(saved: saved, count: list.length));
    } catch (error) {
      emit(DataError(_describe(error)));
    }
  }

  static String _describe(Object error) => switch (error) {
    JournalScanException(:final message) => message,
    _ => error.toString(),
  };
}
