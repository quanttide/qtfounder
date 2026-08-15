import 'package:equatable/equatable.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../models/chapter.dart';
import '../../models/workflow.dart';
import '../../repositories/chapter_repository.dart';

// Events
abstract class WorkflowEvent extends Equatable {
  const WorkflowEvent();

  @override
  List<Object?> get props => [];
}

class LoadWorkflow extends WorkflowEvent {
  const LoadWorkflow();
}

class AdvanceChapter extends WorkflowEvent {
  final String chapterId;
  final String targetStageId;

  const AdvanceChapter(this.chapterId, this.targetStageId);

  @override
  List<Object?> get props => [chapterId, targetStageId];
}

class SelectStage extends WorkflowEvent {
  final String stageId;

  const SelectStage(this.stageId);

  @override
  List<Object?> get props => [stageId];
}

// States
abstract class WorkflowState extends Equatable {
  const WorkflowState();

  @override
  List<Object?> get props => [];
}

class WorkflowInitial extends WorkflowState {
  const WorkflowInitial();
}

class WorkflowLoading extends WorkflowState {
  const WorkflowLoading();
}

class WorkflowLoaded extends WorkflowState {
  final Workflow workflow;
  final String? selectedStageId;
  final bool isAdvancing;

  const WorkflowLoaded({
    required this.workflow,
    this.selectedStageId,
    this.isAdvancing = false,
  });

  WorkflowLoaded copyWith({
    Workflow? workflow,
    String? selectedStageId,
    bool? isAdvancing,
  }) {
    return WorkflowLoaded(
      workflow: workflow ?? this.workflow,
      selectedStageId: selectedStageId ?? this.selectedStageId,
      isAdvancing: isAdvancing ?? this.isAdvancing,
    );
  }

  /// 获取当前选中的阶段
  Stage? get selectedStage {
    if (selectedStageId == null) return null;
    try {
      return workflow.stages.firstWhere((s) => s.id == selectedStageId);
    } catch (_) {
      return null;
    }
  }

  /// 获取指定阶段的章节列表
  List<Chapter> getChaptersInStage(String stageId) {
    return workflow.getChaptersInStage(stageId);
  }

  @override
  List<Object?> get props => [workflow, selectedStageId, isAdvancing];
}

class WorkflowError extends WorkflowState {
  final String message;

  const WorkflowError({required this.message});

  @override
  List<Object?> get props => [message];
}

// Bloc
class WorkflowBloc extends Bloc<WorkflowEvent, WorkflowState> {
  final ChapterRepository _repository;

  WorkflowBloc({required ChapterRepository repository})
      : _repository = repository,
        super(const WorkflowInitial()) {
    on<LoadWorkflow>(_onLoadWorkflow);
    on<AdvanceChapter>(_onAdvanceChapter);
    on<SelectStage>(_onSelectStage);
  }

  Future<void> _onLoadWorkflow(
    LoadWorkflow event,
    Emitter<WorkflowState> emit,
  ) async {
    emit(const WorkflowLoading());
    try {
      final chapters = await _repository.getChapters();
      final workflow = _buildWorkflow(chapters);
      emit(WorkflowLoaded(workflow: workflow));
    } catch (e) {
      emit(WorkflowError(message: e.toString()));
    }
  }

  Future<void> _onAdvanceChapter(
    AdvanceChapter event,
    Emitter<WorkflowState> emit,
  ) async {
    final currentState = state;
    if (currentState is! WorkflowLoaded) return;
    
    emit(currentState.copyWith(isAdvancing: true));
    try {
      await _repository.moveChapter(event.chapterId, event.targetStageId);
      // 重新加载工作流
      add(const LoadWorkflow());
    } catch (e) {
      emit(WorkflowError(message: e.toString()));
    }
  }

  void _onSelectStage(
    SelectStage event,
    Emitter<WorkflowState> emit,
  ) async {
    final currentState = state;
    if (currentState is WorkflowLoaded) {
      emit(currentState.copyWith(selectedStageId: event.stageId));
    }
  }

  /// 从章节列表构建工作流
  Workflow _buildWorkflow(List<Chapter> chapters) {
    final stages = [
      const Stage(
        id: '0_日志',
        name: '0_日志',
        semantics: '动机心境',
        order: 0,
      ),
      const Stage(
        id: '1_灵感',
        name: '1_灵感',
        semantics: '源头',
        order: 1,
      ),
      const Stage(
        id: '2_脚本',
        name: '2_脚本',
        semantics: '素材',
        order: 2,
      ),
      const Stage(
        id: '3_初稿',
        name: '3_初稿',
        semantics: '成文',
        order: 3,
      ),
      const Stage(
        id: '4_改稿',
        name: '4_改稿',
        semantics: '定稿',
        order: 4,
      ),
    ].map((stage) {
      final stageChapters = chapters
          .where((c) => c.stageId == stage.id)
          .toList()
        ..sort((a, b) => a.sortOrder.compareTo(b.sortOrder));
      return stage.copyWith(chapters: stageChapters);
    }).toList();

    return Workflow(stages: stages);
  }
}