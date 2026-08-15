/// 创作职能页——创作工作流显性视图
/// 流程条（0_日志→1_灵感→2_脚本→3_初稿→4_改稿）+ 阶段文件挂载 + 推进/新建/编辑
library;

import 'dart:io';

import 'package:flutter/material.dart';

import '../repositories/asset_catalog_engine.dart';
import '../models/asset_catalog.dart';

class CreateScreen extends StatefulWidget {
  final String? dataSourceRoot; // 测试/定制注入

  const CreateScreen({super.key, this.dataSourceRoot});

  @override
  State<CreateScreen> createState() => _CreateScreenState();
}

class _CreateScreenState extends State<CreateScreen> {
  AssetCatalog? _catalog;
  String? _selectedNovel;
  String? _error;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final engine = AssetCatalogEngine(
        contractAssetPath: 'assets/contracts/fiction.yaml',
        dataSourceRoot: widget.dataSourceRoot,
      );
      final catalog = await engine.load();
      setState(() {
        _catalog = catalog;
        _selectedNovel ??= catalog.nodes.isNotEmpty ? catalog.nodes.first.name : null;
      });
    } catch (e) {
      setState(() => _error = '创作页加载失败：$e');
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_error != null) {
      return Padding(
        padding: const EdgeInsets.all(28),
        child: Text(_error!, style: const TextStyle(color: Color(0xFFEF4444))),
      );
    }
    if (_catalog == null) {
      return const Center(child: CircularProgressIndicator());
    }
    final catalog = _catalog!;
    final novel = catalog.nodes.firstWhere((n) => n.name == _selectedNovel);
    return Padding(
      padding: const EdgeInsets.fromLTRB(28, 28, 28, 0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text('创作',
              style: TextStyle(fontSize: 24, fontWeight: FontWeight.w700, color: Color(0xFF1E293B))),
          const SizedBox(height: 4),
          Text('创作工作流——${novel.name}',
              style: const TextStyle(fontSize: 13, color: Color(0xFF94A3B8))),
          const SizedBox(height: 20),
          _FlowBar(stages: novel.children),
          const SizedBox(height: 16),
          Expanded(
            child: ListView(
              children: novel.children.map((stage) => _StageCard(
                    stage: stage,
                    onAdvance: (file) => _advanceFile(stage, file),
                    onNew: () => _newFile(stage),
                    onEdit: (file) => _editFile(stage, file),
                  )).toList(),
            ),
          ),
        ],
      ),
    );
  }

  /// 推进：文件移动到下一阶段目录
  Future<void> _advanceFile(CatalogNode stage, CatalogFile file) async {
    final stageIndex = stage.name.split('_').first;
    final nextIndex = int.parse(stageIndex) + 1;
    final nextName = '${nextIndex}_';
    final nextPath = file.path.replaceRange(
        file.path.lastIndexOf(Platform.pathSeparator) + 1,
        file.path.length,
        '');
    final targetDir = nextPath.replaceFirst(stage.name, nextName);
    try {
      await File(file.path).rename('$targetDir/${file.name}');
      await _load();
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('${file.title} 已推进到 $nextName 阶段')),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('推进失败：$e')),
        );
      }
    }
  }

  /// 新建：在当前阶段创建文件
  Future<void> _newFile(CatalogNode stage) async {
    final controller = TextEditingController();
    final name = await showDialog<String>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text('新建文件（${stage.name}）'),
        content: TextField(
          controller: controller,
          decoration: const InputDecoration(hintText: '文件名（如 新章节）'),
        ),
        actions: [
          TextButton(onPressed: () => Navigator.pop(ctx), child: const Text('取消')),
          TextButton(
            onPressed: () => Navigator.pop(ctx, controller.text),
            child: const Text('创建'),
          ),
        ],
      ),
    );
    if (name == null || name.trim().isEmpty) return;
    // 阶段目录路径：从已有文件推导
    final samplePath = stage.files.isNotEmpty
        ? stage.files.first.path
        : _stageDir(stage);
    if (samplePath == null) return;
    final dir = samplePath.substring(0, samplePath.lastIndexOf(Platform.pathSeparator));
    try {
      final file = File('$dir/${name.trim()}.md');
      if (!await file.exists()) {
        await file.writeAsString('# $name\n');
      }
      await _load();
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('创建失败：$e')));
      }
    }
  }

  String? _stageDir(CatalogNode stage) {
    // 无文件时通过契约 root 推导（默认路径）
    final home = Platform.environment['HOME'] ?? '';
    return '$home/repos/quanttide-founder/assets/fiction/$_selectedNovel/${stage.name}';
  }

  /// 编辑：写作视图（简单文本编辑 + 保存）
  Future<void> _editFile(CatalogNode stage, CatalogFile file) async {
    String content;
    try {
      content = await File(file.path).readAsString();
    } catch (e) {
      content = '# ${file.title}\n';
    }
    final controller = TextEditingController(text: content);
    if (!mounted) return;
    final saved = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text('写作：${file.title}'),
        content: SizedBox(
          width: 600,
          height: 400,
          child: TextField(
            controller: controller,
            maxLines: null,
            expands: true,
            textAlignVertical: TextAlignVertical.top,
            decoration: const InputDecoration(
              border: OutlineInputBorder(),
              hintText: '正文…',
            ),
          ),
        ),
        actions: [
          TextButton(onPressed: () => Navigator.pop(ctx, false), child: const Text('取消')),
          TextButton(
            onPressed: () => Navigator.pop(ctx, true),
            child: const Text('保存'),
          ),
        ],
      ),
    );
    if (saved == true) {
      final content = controller.text;
      try {
        await File(file.path).writeAsString(content);
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('${file.title} 已保存')),
          );
        }
      } catch (e) {
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('保存失败：$e')));
        }
      }
    }
  }
}

/// 流程条：五阶段 + 语义 + 计数
class _FlowBar extends StatelessWidget {
  final List<CatalogNode> stages;

  const _FlowBar({required this.stages});

  static const _semantics = {
    '0_日志': '动机心境',
    '1_灵感': '源头',
    '2_脚本': '素材',
    '3_初稿': '成文',
    '4_改稿': '定稿',
  };

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        children: [
          for (var i = 0; i < stages.length; i++) ...[
            if (i > 0)
              const Icon(Icons.arrow_forward, size: 14, color: Color(0xFF94A3B8)),
            Expanded(
              child: Column(
                children: [
                  Text(
                    stages[i].name,
                    style: const TextStyle(fontSize: 12, fontWeight: FontWeight.w700, color: Color(0xFF4F46E5)),
                  ),
                  Text(
                    '${_semantics[stages[i].name] ?? stages[i].label} · ${stages[i].files.length}',                    style: const TextStyle(fontSize: 10, color: Color(0xFF94A3B8)),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }
}

/// 阶段卡片：文件列表 + 新建 + 推进
class _StageCard extends StatelessWidget {
  final CatalogNode stage;
  final void Function(CatalogFile file) onAdvance;
  final VoidCallback onNew;
  final void Function(CatalogFile file) onEdit;

  const _StageCard({
    required this.stage,
    required this.onAdvance,
    required this.onNew,
    required this.onEdit,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      margin: const EdgeInsets.only(bottom: 10),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
            child: Row(
              children: [
                const Icon(Icons.folder_outlined, size: 16, color: Color(0xFF4F46E5)),
                const SizedBox(width: 6),
                Text(
                  stage.name,
                  style: const TextStyle(fontSize: 13, fontWeight: FontWeight.w700, color: Color(0xFF1E293B)),
                ),
                const Spacer(),
                TextButton.icon(
                  onPressed: onNew,
                  icon: const Icon(Icons.add, size: 14),
                  label: const Text('新建', style: TextStyle(fontSize: 11)),
                ),
              ],
            ),
          ),
          ...stage.files.map((file) => Padding(
                padding: const EdgeInsets.only(left: 16, right: 8),
                child: Row(
                  children: [
                    const Icon(Icons.description_outlined, size: 14, color: Color(0xFF64748B)),
                    const SizedBox(width: 6),
                    Expanded(
                      child: InkWell(
                        onTap: () => onEdit(file),
                        child: Text(
                          file.title,
                          style: const TextStyle(fontSize: 12, color: Color(0xFF475569)),
                        ),
                      ),
                    ),
                    if (_canAdvance) ...[
                      TextButton.icon(
                        onPressed: () => onAdvance(file),
                        icon: const Icon(Icons.arrow_forward, size: 12),
                        label: Text('推进到$_nextName', style: const TextStyle(fontSize: 10)),
                      ),
                    ],
                  ],
                ),
              )),
          const SizedBox(height: 6),
        ],
      ),
    );
  }

  bool get _canAdvance {
    final idx = int.tryParse(stage.name.split('_').first) ?? 99;
    return idx < 4; // 改稿（4）为终点
  }

  String get _nextName => '${(int.parse(stage.name.split('_').first) + 1)}_';
}
