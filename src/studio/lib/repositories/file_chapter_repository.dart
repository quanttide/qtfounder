import 'dart:io';

import 'package:path/path.dart' as path;

import '../models/chapter.dart';
import 'chapter_repository.dart';

/// 基于文件系统的章节数据访问实现
class FileChapterRepository implements ChapterRepository {
  final String basePath;
  final String novelId;

  FileChapterRepository({
    required this.basePath,
    this.novelId = '职场言情',
  });

  /// 获取小说根目录
  String get _novelPath => path.join(basePath, novelId);

  /// 阶段目录映射
  final Map<String, String> _stageDirectories = {
    '0_日志': '0_日志',
    '1_灵感': '1_灵感',
    '2_脚本': '2_脚本',
    '3_初稿': '3_初稿',
    '4_改稿': '4_改稿',
  };

  @override
  Future<List<Chapter>> getChapters() async {
    final chapters = <Chapter>[];
    
    for (final entry in _stageDirectories.entries) {
      final stageId = entry.key;
      final stageDir = path.join(_novelPath, entry.value);
      final dir = Directory(stageDir);
      
      if (await dir.exists()) {
        await for (final entity in dir.list()) {
          if (entity is File && entity.path.endsWith('.md')) {
            final chapter = await _fileToChapter(entity, stageId);
            if (chapter != null) {
              chapters.add(chapter);
            }
          }
        }
      }
    }
    
    return chapters;
  }

  @override
  Future<Chapter?> getChapter(String id) async {
    final chapters = await getChapters();
    try {
      return chapters.firstWhere((c) => c.id == id);
    } catch (_) {
      return null;
    }
  }

  @override
  Future<String> getChapterContent(String id) async {
    final chapter = await getChapter(id);
    if (chapter == null) {
      throw Exception('Chapter not found: $id');
    }
    
    final file = File(chapter.path);
    if (await file.exists()) {
      return await file.readAsString();
    }
    
    return '';
  }

  @override
  Future<void> saveChapter(String id, String content) async {
    final chapter = await getChapter(id);
    if (chapter == null) {
      throw Exception('Chapter not found: $id');
    }
    
    final file = File(chapter.path);
    await file.writeAsString(content);
  }

  @override
  Future<Chapter> createChapter(String stageId, String title) async {
    final stageDir = _stageDirectories[stageId];
    if (stageDir == null) {
      throw Exception('Invalid stage: $stageId');
    }
    
    final dirPath = path.join(_novelPath, stageDir);
    final dir = Directory(dirPath);
    
    if (!await dir.exists()) {
      await dir.create(recursive: true);
    }
    
    // 生成文件名
    final fileName = _generateFileName(dirPath, title);
    final filePath = path.join(dirPath, fileName);
    
    // 创建文件
    final file = File(filePath);
    await file.writeAsString('# $title\n\n');
    
    // 返回新创建的章节
    return Chapter(
      id: fileName.replaceAll('.md', ''),
      title: title,
      stageId: stageId,
      path: filePath,
      createdAt: DateTime.now(),
      updatedAt: DateTime.now(),
    );
  }

  @override
  Future<void> deleteChapter(String id) async {
    final chapter = await getChapter(id);
    if (chapter == null) {
      throw Exception('Chapter not found: $id');
    }
    
    final file = File(chapter.path);
    if (await file.exists()) {
      await file.delete();
    }
  }

  @override
  Future<void> moveChapter(String id, String targetStageId) async {
    final chapter = await getChapter(id);
    if (chapter == null) {
      throw Exception('Chapter not found: $id');
    }
    
    final targetStageDir = _stageDirectories[targetStageId];
    if (targetStageDir == null) {
      throw Exception('Invalid target stage: $targetStageId');
    }
    
    final targetDirPath = path.join(_novelPath, targetStageDir);
    final targetDir = Directory(targetDirPath);
    
    if (!await targetDir.exists()) {
      await targetDir.create(recursive: true);
    }
    
    final targetPath = path.join(targetDirPath, path.basename(chapter.path));
    
    // 移动文件
    final file = File(chapter.path);
    await file.rename(targetPath);
  }

  @override
  Future<void> renameChapter(String id, String newTitle) async {
    final chapter = await getChapter(id);
    if (chapter == null) {
      throw Exception('Chapter not found: $id');
    }
    
    final oldFile = File(chapter.path);
    final directory = path.dirname(chapter.path);
    final newFileName = _generateFileName(directory, newTitle);
    final newFilePath = path.join(directory, newFileName);
    
    // 重命名文件
    await oldFile.rename(newFilePath);
    
    // 更新文件内容标题
    final newFile = File(newFilePath);
    if (await newFile.exists()) {
      final content = await newFile.readAsString();
      final updatedContent = content.replaceFirst(
        RegExp(r'^# .+'),
        '# $newTitle',
      );
      await newFile.writeAsString(updatedContent);
    }
  }

  /// 将文件转换为章节对象
  Future<Chapter?> _fileToChapter(File file, String stageId) async {
    try {
      final fileName = path.basenameWithoutExtension(file.path);
      final stat = await file.stat();
      
      // 解析文件名获取标题
      final title = _parseTitleFromFileName(fileName);
      
      // 获取文件内容计算字数
      final content = await file.readAsString();
      final wordCount = _countWords(content);
      
      return Chapter(
        id: fileName,
        title: title,
        stageId: stageId,
        path: file.path,
        createdAt: stat.modified,
        updatedAt: stat.modified,
        wordCount: wordCount,
      );
    } catch (e) {
      return null;
    }
  }

  /// 从文件名解析标题
  String _parseTitleFromFileName(String fileName) {
    // 移除编号前缀，如 "1_1_" -> ""
    final parts = fileName.split('_');
    if (parts.length >= 3) {
      // 移除前两部分（编号），剩余部分作为标题
      return parts.sublist(2).join('_');
    }
    return fileName;
  }

  /// 计算字数
  int _countWords(String content) {
    // 移除 Markdown 标记
    final cleanContent = content
        .replaceAll(RegExp(r'#+\s'), '')  // 标题
        .replaceAll(RegExp(r'\*\*.*?\*\*'), '')  // 粗体
        .replaceAll(RegExp(r'\*.*?\*'), '')  // 斜体
        .replaceAll(RegExp(r'\[.*?\]\(.*?\)'), '')  // 链接
        .replaceAll(RegExp(r'!\[.*?\]\(.*?\)'), '')  // 图片
        .replaceAll(RegExp(r'```[\\s\\S]*?```'), '')  // 代码块
        .replaceAll(RegExp(r'`.*?`'), '')  // 行内代码
        .replaceAll(RegExp(r'^\\s*[-*+]\\s', multiLine: true), '')  // 列表
        .replaceAll(RegExp(r'^\\s*\\d+\\.\\s', multiLine: true), '')  // 有序列表
        .replaceAll(RegExp(r'^\\s*>\\s', multiLine: true), '')  // 引用
        .replaceAll(RegExp(r'^\\s*---\\s*$', multiLine: true), '')  // 分割线
        .replaceAll(RegExp(r'\\n+'), ' ')  // 换行
        .trim();
    
    if (cleanContent.isEmpty) {
      return 0;
    }
    
    // 简单的字数统计（按空格分隔）
    return cleanContent.split(RegExp(r'\s+')).length;
  }

  /// 生成文件名（避免重复）
  String _generateFileName(String directoryPath, String title) {
    final sanitizedTitle = title
        .replaceAll(RegExp(r'[<>:"/\\|?*]'), '')
        .replaceAll(' ', '_');
    
    var fileName = '$sanitizedTitle.md';
    var counter = 1;
    
    while (File(path.join(directoryPath, fileName)).existsSync()) {
      fileName = '${sanitizedTitle}_$counter.md';
      counter++;
    }
    
    return fileName;
  }
}