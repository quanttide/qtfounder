/// 思考页 Bloc——情绪结构化处理器（思考云原型）。
///
/// 闭环：日志列表 → 选择某天 → AI 四分类（事实/感受/需要/行动）→
/// 逐条审阅（可忽略）→ 采纳沉淀到日志（追加结构化段，正文不动）。
library;

import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../models/emotion.dart';
import '../../repositories/emotion_repository.dart';
import '../../services/llm_client.dart';

// Events
abstract class ThinkEvent extends Equatable {
  const ThinkEvent();

  @override
  List<Object?> get props => [];
}

/// 加载日志列表
class LoadJournals extends ThinkEvent {
  const LoadJournals();

  @override
  List<Object?> get props => [];
}

/// 选择某天日志（读取内容 + 复用缓存分析结果）
class SelectJournal extends ThinkEvent {
  final String date;

  const SelectJournal(this.date);

  @override
  List<Object?> get props => [date];
}

/// AI 整理（有缓存且未强制时复用）
class AnalyzeJournal extends ThinkEvent {
  final bool force;

  const AnalyzeJournal({this.force = false});

  @override
  List<Object?> get props => [force];
}

/// 忽略某条目（负反馈：从结果中移除，不采纳）
class DismissEntry extends ThinkEvent {
  final EmotionEntry entry;

  const DismissEntry(this.entry);

  @override
  List<Object?> get props => [entry];
}

/// 采纳到日志：把四分类结果追加为 `## 结构化（思考云）` 段
class AdoptStructure extends ThinkEvent {
  const AdoptStructure();

  @override
  List<Object?> get props => [];
}

// States
class ThinkState extends Equatable {
  final List<JournalEntry> journals;
  final String? selectedDate;
  final String? journalContent;
  final EmotionAnalysis? analysis;
  final bool isLoading;
  final bool isAdopting;
  final String? error;

  const ThinkState({
    this.journals = const [],
    this.selectedDate,
    this.journalContent,
    this.analysis,
    this.isLoading = false,
    this.isAdopting = false,
    this.error,
  });

  const ThinkState.initial()
      : journals = const [],
        selectedDate = null,
        journalContent = null,
        analysis = null,
        isLoading = false,
        isAdopting = false,
        error = null;

  ThinkState copyWith({
    List<JournalEntry>? journals,
    String? selectedDate,
    String? journalContent,
    EmotionAnalysis? analysis,
    bool? isLoading,
    bool? isAdopting,
    String? error,
    bool clearAnalysis = false,
    bool clearError = false,
  }) {
    return ThinkState(
      journals: journals ?? this.journals,
      selectedDate: selectedDate ?? this.selectedDate,
      journalContent: journalContent ?? this.journalContent,
      analysis: clearAnalysis ? null : (analysis ?? this.analysis),
      isLoading: isLoading ?? this.isLoading,
      isAdopting: isAdopting ?? this.isAdopting,
      error: clearError ? null : (error ?? this.error),
    );
  }

  @override
  List<Object?> get props => [
        journals,
        selectedDate,
        journalContent,
        analysis,
        isLoading,
        isAdopting,
        error,
      ];
}

// Bloc
class ThinkBloc extends Bloc<ThinkEvent, ThinkState> {
  final JournalRepository _journalRepository;
  final EmotionAnalysisRepository _analysisRepository;
  final LLMClient _llm;
  final Set<String> _pendingDates = {}; // 分析中去重

  ThinkBloc({
    required JournalRepository journalRepository,
    required EmotionAnalysisRepository analysisRepository,
    required LLMClient llm,
  })  : _journalRepository = journalRepository,
        _analysisRepository = analysisRepository,
        _llm = llm,
        super(const ThinkState.initial()) {
    on<LoadJournals>(_onLoadJournals);
    on<SelectJournal>(_onSelectJournal);
    on<AnalyzeJournal>(_onAnalyzeJournal);
    on<DismissEntry>(_onDismissEntry);
    on<AdoptStructure>(_onAdoptStructure);
  }

  Future<void> _onLoadJournals(
    LoadJournals event,
    Emitter<ThinkState> emit,
  ) async {
    try {
      final journals = await _journalRepository.listJournals();
      emit(state.copyWith(journals: journals, error: null, clearError: false));
    } catch (e) {
      emit(state.copyWith(error: '加载日志失败: $e'));
    }
  }

  Future<void> _onSelectJournal(
    SelectJournal event,
    Emitter<ThinkState> emit,
  ) async {
    final date = event.date;
    final journal = state.journals
        .firstWhere((j) => j.date == date, orElse: () => throw StateError('日志不存在: $date'));
    try {
      final content = await _journalRepository.readJournal(journal.path);
      final cached = await _analysisRepository.getAnalysis(journal.path, date);
      emit(state.copyWith(
        selectedDate: date,
        journalContent: content,
        analysis: cached,
        isLoading: false,
        error: null,
        clearError: false,
      ));
    } catch (e) {
      emit(state.copyWith(error: '读取日志失败: $e'));
    }
  }

  Future<void> _onAnalyzeJournal(
    AnalyzeJournal event,
    Emitter<ThinkState> emit,
  ) async {
    final date = state.selectedDate;
    final content = state.journalContent;
    if (date == null || content == null) return;
    if (_pendingDates.contains(date)) return; // 去重
    _pendingDates.add(date);

    try {
      final journal = state.journals
          .firstWhere((j) => j.date == date, orElse: () => throw StateError('日志不存在: $date'));

      // 缓存命中且未强制刷新 → 复用
      if (!event.force) {
        final cached = await _analysisRepository.getAnalysis(journal.path, date);
        if (cached != null) {
          emit(state.copyWith(analysis: cached, isLoading: false));
          return;
        }
      }

      emit(state.copyWith(isLoading: true, error: null, clearError: false));
      final analysis = await _llm.structureEmotion(
        journalPath: journal.path,
        date: date,
        content: content,
      );
      await _analysisRepository.saveAnalysis(analysis);
      emit(state.copyWith(analysis: analysis, isLoading: false));
    } catch (e) {
      emit(state.copyWith(isLoading: false, error: '整理失败: $e'));
    } finally {
      _pendingDates.remove(date);
    }
  }

  Future<void> _onDismissEntry(
    DismissEntry event,
    Emitter<ThinkState> emit,
  ) async {
    final analysis = state.analysis;
    if (analysis == null) return;
    final updated = analysis.copyWith(
      entries: analysis.entries
          .where((e) => e != event.entry)
          .toList(),
    );
    try {
      await _analysisRepository.saveAnalysis(updated);
      emit(state.copyWith(analysis: updated));
    } catch (e) {
      emit(state.copyWith(error: '保存失败: $e'));
    }
  }

  Future<void> _onAdoptStructure(
    AdoptStructure event,
    Emitter<ThinkState> emit,
  ) async {
    final analysis = state.analysis;
    final date = state.selectedDate;
    if (analysis == null || date == null) return;

    try {
      emit(state.copyWith(isAdopting: true, error: null, clearError: false));
      final journal = state.journals
          .firstWhere((j) => j.date == date, orElse: () => throw StateError('日志不存在: $date'));
      await _journalRepository.appendStructureSection(journal.path, analysis);
      // 标记已处理并刷新列表
      final journals = state.journals
          .map((j) => j.date == date ? JournalEntry(date: j.date, path: j.path, processed: true) : j)
          .toList();
      emit(state.copyWith(journals: journals, isAdopting: false));
    } catch (e) {
      emit(state.copyWith(isAdopting: false, error: '采纳失败: $e'));
    }
  }
}
