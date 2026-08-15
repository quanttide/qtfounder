import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../models/chapter.dart';
import '../../repositories/chapter_repository.dart';

// Events
abstract class ChapterListEvent extends Equatable {
  const ChapterListEvent();

  @override
  List<Object?> get props => [];
}

class LoadChapters extends ChapterListEvent {
  const LoadChapters();
}

class SelectChapter extends ChapterListEvent {
  final String chapterId;

  const SelectChapter(this.chapterId);

  @override
  List<Object?> get props => [chapterId];
}

class CreateChapter extends ChapterListEvent {
  final String stageId;
  final String title;

  const CreateChapter(this.stageId, this.title);

  @override
  List<Object?> get props => [stageId, title];
}

class DeleteChapter extends ChapterListEvent {
  final String chapterId;

  const DeleteChapter(this.chapterId);

  @override
  List<Object?> get props => [chapterId];
}

class MoveChapter extends ChapterListEvent {
  final String chapterId;
  final String targetStageId;

  const MoveChapter(this.chapterId, this.targetStageId);

  @override
  List<Object?> get props => [chapterId, targetStageId];
}

class RenameChapter extends ChapterListEvent {
  final String chapterId;
  final String newTitle;

  const RenameChapter(this.chapterId, this.newTitle);

  @override
  List<Object?> get props => [chapterId, newTitle];
}

class ToggleFavorite extends ChapterListEvent {
  final String chapterId;

  const ToggleFavorite(this.chapterId);

  @override
  List<Object?> get props => [chapterId];
}

// States
abstract class ChapterListState extends Equatable {
  const ChapterListState();

  @override
  List<Object?> get props => [];
}

class ChapterListInitial extends ChapterListState {
  const ChapterListInitial();
}

class ChapterListLoading extends ChapterListState {
  const ChapterListLoading();
}

class ChapterListLoaded extends ChapterListState {
  final List<Chapter> chapters;
  final String? selectedChapterId;
  final String? searchQuery;

  const ChapterListLoaded({
    required this.chapters,
    this.selectedChapterId,
    this.searchQuery,
  });

  ChapterListLoaded copyWith({
    List<Chapter>? chapters,
    String? selectedChapterId,
    String? searchQuery,
  }) {
    return ChapterListLoaded(
      chapters: chapters ?? this.chapters,
      selectedChapterId: selectedChapterId ?? this.selectedChapterId,
      searchQuery: searchQuery ?? this.searchQuery,
    );
  }

  /// 获取当前选中的章节
  Chapter? get selectedChapter {
    if (selectedChapterId == null) return null;
    try {
      return chapters.firstWhere((c) => c.id == selectedChapterId);
    } catch (_) {
      return null;
    }
  }

  /// 按阶段分组的章节
  Map<String, List<Chapter>> get chaptersByStage {
    final map = <String, List<Chapter>>{};
    for (final chapter in chapters) {
      map.putIfAbsent(chapter.stageId, () => []).add(chapter);
    }
    return map;
  }

  /// 搜索过滤后的章节
  List<Chapter> get filteredChapters {
    if (searchQuery == null || searchQuery!.isEmpty) {
      return chapters;
    }
    final query = searchQuery!.toLowerCase();
    return chapters
        .where((c) => c.title.toLowerCase().contains(query))
        .toList();
  }

  /// 最近编辑的章节（最多10个）
  List<Chapter> get recentChapters {
    final sorted = List<Chapter>.from(chapters)
      ..sort((a, b) => b.updatedAt.compareTo(a.updatedAt));
    return sorted.take(10).toList();
  }

  /// 收藏的章节
  List<Chapter> get favoriteChapters {
    return chapters.where((c) => c.isFavorite).toList();
  }

  @override
  List<Object?> get props => [chapters, selectedChapterId, searchQuery];
}

class ChapterListError extends ChapterListState {
  final String message;

  const ChapterListError({required this.message});

  @override
  List<Object?> get props => [message];
}

// Bloc
class ChapterListBloc extends Bloc<ChapterListEvent, ChapterListState> {
  final ChapterRepository _repository;

  ChapterListBloc({required ChapterRepository repository})
      : _repository = repository,
        super(const ChapterListInitial()) {
    on<LoadChapters>(_onLoadChapters);
    on<SelectChapter>(_onSelectChapter);
    on<CreateChapter>(_onCreateChapter);
    on<DeleteChapter>(_onDeleteChapter);
    on<MoveChapter>(_onMoveChapter);
    on<RenameChapter>(_onRenameChapter);
    on<ToggleFavorite>(_onToggleFavorite);
  }

  Future<void> _onLoadChapters(
    LoadChapters event,
    Emitter<ChapterListState> emit,
  ) async {
    emit(const ChapterListLoading());
    try {
      final chapters = await _repository.getChapters();
      emit(ChapterListLoaded(chapters: chapters));
    } catch (e) {
      emit(ChapterListError(message: e.toString()));
    }
  }

  Future<void> _onSelectChapter(
    SelectChapter event,
    Emitter<ChapterListState> emit,
  ) async {
    final currentState = state;
    if (currentState is ChapterListLoaded) {
      emit(currentState.copyWith(selectedChapterId: event.chapterId));
    }
  }

  Future<void> _onCreateChapter(
    CreateChapter event,
    Emitter<ChapterListState> emit,
  ) async {
    try {
      await _repository.createChapter(event.stageId, event.title);
      add(const LoadChapters());
    } catch (e) {
      emit(ChapterListError(message: e.toString()));
    }
  }

  Future<void> _onDeleteChapter(
    DeleteChapter event,
    Emitter<ChapterListState> emit,
  ) async {
    try {
      await _repository.deleteChapter(event.chapterId);
      add(const LoadChapters());
    } catch (e) {
      emit(ChapterListError(message: e.toString()));
    }
  }

  Future<void> _onMoveChapter(
    MoveChapter event,
    Emitter<ChapterListState> emit,
  ) async {
    try {
      await _repository.moveChapter(event.chapterId, event.targetStageId);
      add(const LoadChapters());
    } catch (e) {
      emit(ChapterListError(message: e.toString()));
    }
  }

  Future<void> _onRenameChapter(
    RenameChapter event,
    Emitter<ChapterListState> emit,
  ) async {
    try {
      await _repository.renameChapter(event.chapterId, event.newTitle);
      add(const LoadChapters());
    } catch (e) {
      emit(ChapterListError(message: e.toString()));
    }
  }

  Future<void> _onToggleFavorite(
    ToggleFavorite event,
    Emitter<ChapterListState> emit,
  ) async {
    final currentState = state;
    if (currentState is ChapterListLoaded) {
      final chapters = currentState.chapters.map((c) {
        if (c.id == event.chapterId) {
          return c.copyWith(isFavorite: !c.isFavorite);
        }
        return c;
      }).toList();
      emit(currentState.copyWith(chapters: chapters));
    }
  }
}