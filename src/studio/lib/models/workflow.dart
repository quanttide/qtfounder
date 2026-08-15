import 'package:equatable/equatable.dart';

import 'chapter.dart';

/// 工作流模型
class Workflow extends Equatable {
  final List<Stage> stages;
  final String currentStageId;

  const Workflow({
    required this.stages,
    this.currentStageId = '',
  });

  Workflow copyWith({
    List<Stage>? stages,
    String? currentStageId,
  }) {
    return Workflow(
      stages: stages ?? this.stages,
      currentStageId: currentStageId ?? this.currentStageId,
    );
  }

  /// 获取当前阶段
  Stage? get currentStage {
    try {
      return stages.firstWhere((s) => s.id == currentStageId);
    } catch (_) {
      return null;
    }
  }

  /// 获取指定阶段的章节列表
  List<Chapter> getChaptersInStage(String stageId) {
    try {
      final stage = stages.firstWhere((s) => s.id == stageId);
      return stage.chapters;
    } catch (_) {
      return [];
    }
  }

  @override
  List<Object?> get props => [stages, currentStageId];
}

/// 阶段模型
class Stage extends Equatable {
  final String id;
  final String name;
  final String semantics;
  final int order;
  final List<Chapter> chapters;

  const Stage({
    required this.id,
    required this.name,
    required this.semantics,
    required this.order,
    this.chapters = const [],
  });

  Stage copyWith({
    String? id,
    String? name,
    String? semantics,
    int? order,
    List<Chapter>? chapters,
  }) {
    return Stage(
      id: id ?? this.id,
      name: name ?? this.name,
      semantics: semantics ?? this.semantics,
      order: order ?? this.order,
      chapters: chapters ?? this.chapters,
    );
  }

  /// 章节数量
  int get chapterCount => chapters.length;

  /// 是否可以推进（不是最后一个阶段）
  bool get canAdvance => order < 4;

  /// 下一个阶段ID
  String? get nextStageId {
    if (!canAdvance) return null;
    final nextOrder = order + 1;
    final stageNames = ['0_日志', '1_灵感', '2_脚本', '3_初稿', '4_改稿'];
    if (nextOrder < stageNames.length) {
      return stageNames[nextOrder];
    }
    return null;
  }

  @override
  List<Object?> get props => [id, name, semantics, order, chapters];
}