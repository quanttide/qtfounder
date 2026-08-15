import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../models/analysis.dart';
import '../../repositories/analysis_repository.dart';
import '../../repositories/chapter_repository.dart';
import '../../services/llm_client.dart';

// Events
abstract class AnalyzeEvent extends Equatable {
  const AnalyzeEvent();

  @override
  List<Object?> get props => [];
}

/// 分析章节（有缓存且未过期时复用）
class AnalyzeChapter extends AnalyzeEvent {
  final String chapterId;

  const AnalyzeChapter(this.chapterId);

  @override
  List<Object?> get props => [chapterId];
}

/// 强制重新分析（忽略缓存）
class RefreshAnalysis extends AnalyzeEvent {
  final String chapterId;

  const RefreshAnalysis(this.chapterId);

  @override
  List<Object?> get props => [chapterId];
}

/// 忽略建议（负反馈，后续分析不再提出）
class IgnoreSuggestion extends AnalyzeEvent {
  final String suggestionId;

  const IgnoreSuggestion(this.suggestionId);

  @override
  List<Object?> get props => [suggestionId];
}

/// 清除分析状态（切换章节时）
class ClearAnalysis extends AnalyzeEvent {
  const ClearAnalysis();

  @override
  List<Object?> get props => [];
}

// States
class AnalysisState extends Equatable {
  final ChapterAnalysis? analysis;
  final bool isLoading;
  final String? error;
  final String? analyzingChapterId;

  const AnalysisState({
    this.analysis,
    this.isLoading = false,
    this.error,
    this.analyzingChapterId,
  });

  const AnalysisState.initial()
      : analysis = null,
        isLoading = false,
        error = null,
        analyzingChapterId = null;

  AnalysisState copyWith({
    ChapterAnalysis? analysis,
    bool? isLoading,
    String? error,
    String? analyzingChapterId,
    bool clearAnalysis = false,
    bool clearError = false,
  }) {
    return AnalysisState(
      analysis: clearAnalysis ? null : (analysis ?? this.analysis),
      isLoading: isLoading ?? this.isLoading,
      error: clearError ? null : (error ?? this.error),
      analyzingChapterId:
          analyzingChapterId ?? this.analyzingChapterId,
    );
  }

  @override
  List<Object?> get props => [analysis, isLoading, error, analyzingChapterId];
}

// Bloc
class AnalyzeBloc extends Bloc<AnalyzeEvent, AnalysisState> {
  final ChapterRepository _chapterRepository;
  final ChapterAnalysisRepository _analysisRepository;
  final LLMClient _llm;
  final Set<String> _pendingIds = {}; // 分析中去重

  AnalyzeBloc({
    required ChapterRepository chapterRepository,
    required ChapterAnalysisRepository analysisRepository,
    required LLMClient llm,
  })  : _chapterRepository = chapterRepository,
        _analysisRepository = analysisRepository,
        _llm = llm,
        super(const AnalysisState.initial()) {
    on<AnalyzeChapter>(_onAnalyzeChapter);
    on<RefreshAnalysis>(_onRefreshAnalysis);
    on<IgnoreSuggestion>(_onIgnoreSuggestion);
    on<ClearAnalysis>(_onClearAnalysis);
  }

  Future<void> _onAnalyzeChapter(
    AnalyzeChapter event,
    Emitter<AnalysisState> emit,
  ) async {
    await _analyze(event.chapterId, emit, force: false);
  }

  Future<void> _onRefreshAnalysis(
    RefreshAnalysis event,
    Emitter<AnalysisState> emit,
  ) async {
    await _analyze(event.chapterId, emit, force: true);
  }

  Future<void> _analyze(
    String chapterId,
    Emitter<AnalysisState> emit, {
    required bool force,
  }) async {
    if (_pendingIds.contains(chapterId)) return; // 去重
    _pendingIds.add(chapterId);

    try {
      final chapter = await _chapterRepository.getChapter(chapterId);
      if (chapter == null) {
        emit(state.copyWith(error: '章节不存在: $chapterId', clearError: false));
        return;
      }

      // 缓存命中且未强制刷新 → 直接复用
      if (!force) {
        final cached = await _analysisRepository.getAnalysis(chapter.path);
        if (cached != null) {
          emit(state.copyWith(
            analysis: cached,
            isLoading: false,
            analyzingChapterId: chapterId,
          ));
          return;
        }
      }

      emit(state.copyWith(
        isLoading: true,
        error: null,
        analyzingChapterId: chapterId,
      ));

      final content = await _chapterRepository.getChapterContent(chapterId);
      final previous = await _analysisRepository.getAnalysis(chapter.path);

      final analysis = await _llm.analyzeStructure(
        chapterId: chapterId,
        chapterPath: chapter.path,
        content: content,
        previousSuggestions: previous?.ignoredSuggestions ?? const [],
      );
      await _analysisRepository.saveAnalysis(analysis);

      emit(state.copyWith(
        analysis: analysis,
        isLoading: false,
        analyzingChapterId: chapterId,
      ));
    } catch (e) {
      emit(state.copyWith(
        isLoading: false,
        error: '分析失败: $e',
      ));
    } finally {
      _pendingIds.remove(chapterId);
    }
  }

  Future<void> _onIgnoreSuggestion(
    IgnoreSuggestion event,
    Emitter<AnalysisState> emit,
  ) async {
    final analysis = state.analysis;
    if (analysis == null) return;
    final updated = analysis.copyWith(
      ignoredSuggestions: [...analysis.ignoredSuggestions, event.suggestionId],
    );
    try {
      await _analysisRepository.saveAnalysis(updated);
      emit(state.copyWith(analysis: updated));
    } catch (e) {
      emit(state.copyWith(error: '保存忽略记录失败: $e'));
    }
  }

  void _onClearAnalysis(
    ClearAnalysis event,
    Emitter<AnalysisState> emit,
  ) {
    emit(const AnalysisState.initial());
  }
}
