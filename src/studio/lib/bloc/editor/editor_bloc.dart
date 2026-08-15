import 'dart:async';

import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../repositories/chapter_repository.dart';

// Events
abstract class EditorEvent extends Equatable {
  const EditorEvent();

  @override
  List<Object?> get props => [];
}

class LoadContent extends EditorEvent {
  final String chapterId;

  const LoadContent(this.chapterId);

  @override
  List<Object?> get props => [chapterId];
}

class UpdateContent extends EditorEvent {
  final String content;

  const UpdateContent(this.content);

  @override
  List<Object?> get props => [content];
}

class SaveContent extends EditorEvent {
  const SaveContent();
}

class AutoSaveTriggered extends EditorEvent {
  const AutoSaveTriggered();
}

class ClearEditor extends EditorEvent {
  const ClearEditor();
}

// States
enum SaveStatus {
  saved,
  saving,
  unsaved,
  error,
}

class EditorState extends Equatable {
  final String? chapterId;
  final String content;
  final String initialContent;
  final bool isLoading;
  final bool isDirty;
  final SaveStatus saveStatus;
  final int wordCount;
  final String? error;
  final bool isPreviewMode;
  final bool isFullScreen;
  final int cursorPosition;

  const EditorState({
    this.chapterId,
    this.content = '',
    this.initialContent = '',
    this.isLoading = false,
    this.isDirty = false,
    this.saveStatus = SaveStatus.saved,
    this.wordCount = 0,
    this.error,
    this.isPreviewMode = false,
    this.isFullScreen = false,
    this.cursorPosition = 0,
  });

  const EditorState.initial()
      : chapterId = null,
        content = '',
        initialContent = '',
        isLoading = false,
        isDirty = false,
        saveStatus = SaveStatus.saved,
        wordCount = 0,
        error = null,
        isPreviewMode = false,
        isFullScreen = false,
        cursorPosition = 0;

  EditorState copyWith({
    String? chapterId,
    String? content,
    String? initialContent,
    bool? isLoading,
    bool? isDirty,
    SaveStatus? saveStatus,
    int? wordCount,
    String? error,
    bool? isPreviewMode,
    bool? isFullScreen,
    int? cursorPosition,
  }) {
    return EditorState(
      chapterId: chapterId ?? this.chapterId,
      content: content ?? this.content,
      initialContent: initialContent ?? this.initialContent,
      isLoading: isLoading ?? this.isLoading,
      isDirty: isDirty ?? this.isDirty,
      saveStatus: saveStatus ?? this.saveStatus,
      wordCount: wordCount ?? this.wordCount,
      error: error,
      isPreviewMode: isPreviewMode ?? this.isPreviewMode,
      isFullScreen: isFullScreen ?? this.isFullScreen,
      cursorPosition: cursorPosition ?? this.cursorPosition,
    );
  }

  /// 计算字数
  static int calculateWordCount(String text) {
    if (text.isEmpty) return 0;
    
    // 移除 Markdown 标记
    final cleanContent = text
        .replaceAll(RegExp(r'#+\s'), '')
        .replaceAll(RegExp(r'\*\*.*?\*\*'), '')
        .replaceAll(RegExp(r'\*.*?\*'), '')
        .replaceAll(RegExp(r'\[.*?\]\(.*?\)'), '')
        .replaceAll(RegExp(r'!\[.*?\]\(.*?\)'), '')
        .replaceAll(RegExp(r'```.*?```', multiLine: true), '')
        .replaceAll(RegExp(r'`.*?`'), '')
        .replaceAll(RegExp(r'^\s*[-*+]\s', multiLine: true), '')
        .replaceAll(RegExp(r'^\s*\d+\.\s', multiLine: true), '')
        .replaceAll(RegExp(r'^\s*>\s', multiLine: true), '')
        .replaceAll(RegExp(r'^\s*---\s*$', multiLine: true), '')
        .replaceAll(RegExp(r'\n+'), ' ')
        .trim();
    
    if (cleanContent.isEmpty) return 0;
    
    return cleanContent.split(RegExp(r'\s+')).length;
  }

  @override
  List<Object?> get props => [
        chapterId,
        content,
        initialContent,
        isLoading,
        isDirty,
        saveStatus,
        wordCount,
        error,
        isPreviewMode,
        isFullScreen,
        cursorPosition,
      ];
}

// Bloc
class EditorBloc extends Bloc<EditorEvent, EditorState> {
  final ChapterRepository _repository;
  Timer? _autoSaveTimer;

  EditorBloc({required ChapterRepository repository})
      : _repository = repository,
        super(EditorState.initial()) {
    on<LoadContent>(_onLoadContent);
    on<UpdateContent>(_onUpdateContent);
    on<SaveContent>(_onSaveContent);
    on<AutoSaveTriggered>(_onAutoSave);
    on<ClearEditor>(_onClearEditor);
  }

  Future<void> _onLoadContent(
    LoadContent event,
    Emitter<EditorState> emit,
  ) async {
    emit(state.copyWith(isLoading: true, error: null));
    try {
      final content = await _repository.getChapterContent(event.chapterId);
      final wordCount = EditorState.calculateWordCount(content);
      emit(state.copyWith(
        chapterId: event.chapterId,
        content: content,
        initialContent: content,
        isLoading: false,
        isDirty: false,
        saveStatus: SaveStatus.saved,
        wordCount: wordCount,
      ));
    } catch (e) {
      emit(state.copyWith(
        isLoading: false,
        error: e.toString(),
      ));
    }
  }

  void _onUpdateContent(
    UpdateContent event,
    Emitter<EditorState> emit,
  ) {
    final wordCount = EditorState.calculateWordCount(event.content);
    emit(state.copyWith(
      content: event.content,
      isDirty: true,
      saveStatus: SaveStatus.unsaved,
      wordCount: wordCount,
    ));
    _scheduleAutoSave();
  }

  void _scheduleAutoSave() {
    _autoSaveTimer?.cancel();
    _autoSaveTimer = Timer(const Duration(seconds: 3), () {
      add(const AutoSaveTriggered());
    });
  }

  Future<void> _onAutoSave(
    AutoSaveTriggered event,
    Emitter<EditorState> emit,
  ) async {
    if (state.isDirty && state.chapterId != null) {
      add(const SaveContent());
    }
  }

  Future<void> _onSaveContent(
    SaveContent event,
    Emitter<EditorState> emit,
  ) async {
    if (state.chapterId == null) return;
    
    emit(state.copyWith(saveStatus: SaveStatus.saving));
    try {
      await _repository.saveChapter(state.chapterId!, state.content);
      emit(state.copyWith(
        saveStatus: SaveStatus.saved,
        isDirty: false,
        initialContent: state.content,
      ));
    } catch (e) {
      emit(state.copyWith(saveStatus: SaveStatus.error));
    }
  }

  void _onClearEditor(
    ClearEditor event,
    Emitter<EditorState> emit,
  ) {
    _autoSaveTimer?.cancel();
    emit(EditorState.initial());
  }

  @override
  Future<void> close() {
    _autoSaveTimer?.cancel();
    return super.close();
  }
}