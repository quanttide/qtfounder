import 'package:qtfounder_studio/models/chapter.dart';
import 'package:qtfounder_studio/repositories/chapter_repository.dart';

/// Mock ChapterRepository for testing
class MockChapterRepository implements ChapterRepository {
  final List<Chapter> _chapters = [];
  final Map<String, String> _contents = {};

  MockChapterRepository() {
    // 初始化测试数据
    _chapters.addAll([
      Chapter(
        id: 'test_chapter_1',
        title: '测试章节1',
        stageId: '0_日志',
        path: '/test/test_chapter_1.md',
        createdAt: DateTime(2026, 1, 1),
        updatedAt: DateTime(2026, 1, 1),
      ),
      Chapter(
        id: 'test_chapter_2',
        title: '测试章节2',
        stageId: '1_灵感',
        path: '/test/test_chapter_2.md',
        createdAt: DateTime(2026, 1, 2),
        updatedAt: DateTime(2026, 1, 2),
      ),
    ]);
    
    _contents['test_chapter_1'] = '# 测试章节1\n\n这是测试内容。';
    _contents['test_chapter_2'] = '# 测试章节2\n\n更多测试内容。';
  }

  @override
  Future<List<Chapter>> getChapters() async {
    return List.unmodifiable(_chapters);
  }

  @override
  Future<Chapter?> getChapter(String id) async {
    try {
      return _chapters.firstWhere((c) => c.id == id);
    } catch (_) {
      return null;
    }
  }

  @override
  Future<String> getChapterContent(String id) async {
    return _contents[id] ?? '';
  }

  @override
  Future<void> saveChapter(String id, String content) async {
    _contents[id] = content;
  }

  @override
  Future<Chapter> createChapter(String stageId, String title) async {
    final id = 'new_chapter_${_chapters.length + 1}';
    final chapter = Chapter(
      id: id,
      title: title,
      stageId: stageId,
      path: '/test/$id.md',
      createdAt: DateTime.now(),
      updatedAt: DateTime.now(),
    );
    _chapters.add(chapter);
    _contents[id] = '# $title\n\n';
    return chapter;
  }

  @override
  Future<void> deleteChapter(String id) async {
    _chapters.removeWhere((c) => c.id == id);
    _contents.remove(id);
  }

  @override
  Future<void> moveChapter(String id, String targetStageId) async {
    final index = _chapters.indexWhere((c) => c.id == id);
    if (index != -1) {
      _chapters[index] = _chapters[index].copyWith(stageId: targetStageId);
    }
  }

  @override
  Future<void> renameChapter(String id, String newTitle) async {
    final index = _chapters.indexWhere((c) => c.id == id);
    if (index != -1) {
      _chapters[index] = _chapters[index].copyWith(title: newTitle);
    }
  }
}