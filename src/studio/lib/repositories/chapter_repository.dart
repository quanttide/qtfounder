import '../models/chapter.dart';

/// 章节数据访问抽象层
abstract class ChapterRepository {
  /// 获取所有章节
  Future<List<Chapter>> getChapters();

  /// 获取指定章节
  Future<Chapter?> getChapter(String id);

  /// 获取章节内容
  Future<String> getChapterContent(String id);

  /// 保存章节内容
  Future<void> saveChapter(String id, String content);

  /// 创建新章节
  Future<Chapter> createChapter(String stageId, String title);

  /// 删除章节
  Future<void> deleteChapter(String id);

  /// 移动章节到其他阶段
  Future<void> moveChapter(String id, String targetStageId);

  /// 重命名章节
  Future<void> renameChapter(String id, String newTitle);
}