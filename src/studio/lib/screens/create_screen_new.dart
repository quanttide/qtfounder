/// 创作职能页——基于 Bloc 的 AI 原生编辑器实现。
///
/// 布局：章节导航侧边栏 + 编辑器主区域 + 可选 AI 整理面板。
/// 原则：AI 只产出结构（元数据），不改写原文。
library;

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../bloc/analyze/analyze_bloc.dart';
import '../bloc/editor/editor_bloc.dart';
import '../bloc/workflow/workflow_bloc.dart';
import '../models/analysis.dart';
import '../models/chapter.dart';
import '../models/workflow.dart';
import 'analysis_panel.dart';
import 'annotation_overlay.dart';

class CreateScreenNew extends StatefulWidget {
  const CreateScreenNew({super.key});

  @override
  State<CreateScreenNew> createState() => _CreateScreenNewState();
}

class _CreateScreenNewState extends State<CreateScreenNew> {
  bool _showAnalysis = false;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Row(
        children: [
          // 左侧：章节导航侧边栏
          SizedBox(
            width: 280,
            child: _buildChapterNavigator(context),
          ),
          // 中间：编辑器主区域
          Expanded(
            child: _buildEditorArea(context),
          ),
          // 右侧：AI 整理面板（可选）
          if (_showAnalysis) ...[
            BlocBuilder<EditorBloc, EditorState>(
              builder: (context, state) =>
                  AnalysisPanel(chapterId: state.chapterId),
            ),
          ],
        ],
      ),
    );
  }

  /// 构建章节导航侧边栏（阶段分组树）
  Widget _buildChapterNavigator(BuildContext context) {
    return Container(
      color: Colors.white,
      child: Column(
        children: [
          // 搜索框
          _buildSearchBar(context),
          // 章节列表
          Expanded(
            child: _buildChapterList(context),
          ),
        ],
      ),
    );
  }

  /// 构建搜索框
  Widget _buildSearchBar(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      child: TextField(
        decoration: InputDecoration(
          hintText: '搜索章节...',
          prefixIcon: const Icon(Icons.search, size: 20),
          border: OutlineInputBorder(
            borderRadius: BorderRadius.circular(8),
            borderSide: BorderSide(color: Colors.grey.shade300),
          ),
          contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          isDense: true,
        ),
        onChanged: (value) {
          // TODO: 实现搜索功能
        },
      ),
    );
  }

  /// 构建章节列表
  Widget _buildChapterList(BuildContext context) {
    return BlocBuilder<WorkflowBloc, WorkflowState>(
      builder: (context, state) {
        if (state is WorkflowLoading) {
          return const Center(child: CircularProgressIndicator());
        }

        if (state is WorkflowError) {
          return Center(child: Text('加载失败: ${state.message}'));
        }

        if (state is WorkflowLoaded) {
          return _buildStageGroups(context, state.workflow);
        }

        return const SizedBox.shrink();
      },
    );
  }

  /// 构建阶段分组
  Widget _buildStageGroups(BuildContext context, Workflow workflow) {
    return ListView.builder(
      itemCount: workflow.stages.length,
      itemBuilder: (context, index) {
        final stage = workflow.stages[index];
        return _buildStageGroup(context, stage);
      },
    );
  }

  /// 构建单个阶段分组（含阶段区分：语义 + 计数）
  Widget _buildStageGroup(BuildContext context, Stage stage) {
    return ExpansionTile(
      initiallyExpanded: stage.order == 0,
      title: Row(
        children: [
          Text(
            stage.name,
            style: const TextStyle(
              fontSize: 13,
              fontWeight: FontWeight.w600,
            ),
          ),
          const SizedBox(width: 8),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
            decoration: BoxDecoration(
              color: Colors.grey.shade200,
              borderRadius: BorderRadius.circular(10),
            ),
            child: Text(
              '${stage.chapterCount}',
              style: const TextStyle(fontSize: 11),
            ),
          ),
        ],
      ),
      subtitle: Text(
        stage.semantics,
        style: TextStyle(fontSize: 11, color: Colors.grey.shade600),
      ),
      children: stage.chapters.map((chapter) {
        return _buildChapterTile(context, chapter);
      }).toList(),
    );
  }

  /// 构建章节项
  Widget _buildChapterTile(BuildContext context, Chapter chapter) {
    final isSelected = context.read<EditorBloc>().state.chapterId == chapter.id;

    return ListTile(
      dense: true,
      selected: isSelected,
      selectedTileColor: Colors.indigo.shade50,
      title: Text(
        chapter.title,
        style: const TextStyle(fontSize: 12),
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
      subtitle: Text(
        '${chapter.wordCount} 字',
        style: TextStyle(fontSize: 10, color: Colors.grey.shade500),
      ),
      leading: Icon(
        chapter.isFavorite ? Icons.star : Icons.star_border,
        size: 16,
        color: chapter.isFavorite ? Colors.amber : Colors.grey,
      ),
      onTap: () {
        // 切换章节：清除上一章节的分析状态，加载新内容
        context.read<AnalyzeBloc>().add(const ClearAnalysis());
        context.read<EditorBloc>().add(LoadContent(chapter.id));
      },
    );
  }

  /// 构建编辑器主区域
  Widget _buildEditorArea(BuildContext context) {
    return BlocBuilder<EditorBloc, EditorState>(
      builder: (context, state) {
        if (state.isLoading) {
          return const Center(child: CircularProgressIndicator());
        }

        if (state.chapterId == null) {
          return _buildEmptyState();
        }

        return _buildEditor(context, state);
      },
    );
  }

  /// 构建空状态
  Widget _buildEmptyState() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(
            Icons.edit_note,
            size: 64,
            color: Colors.grey.shade400,
          ),
          const SizedBox(height: 16),
          Text(
            '选择一个章节开始写作',
            style: TextStyle(
              fontSize: 16,
              color: Colors.grey.shade600,
            ),
          ),
        ],
      ),
    );
  }

  /// 构建编辑器
  Widget _buildEditor(BuildContext context, EditorState state) {
    return Column(
      children: [
        // 工具栏
        _buildToolbar(context, state),
        // 编辑器内容
        Expanded(
          child: _buildEditorContent(context, state),
        ),
        // 状态栏
        _buildStatusBar(context, state),
      ],
    );
  }

  /// 构建工具栏
  Widget _buildToolbar(BuildContext context, EditorState state) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      decoration: BoxDecoration(
        color: Colors.white,
        border: Border(
          bottom: BorderSide(color: Colors.grey.shade200),
        ),
      ),
      child: Row(
        children: [
          // 保存状态指示器
          _buildSaveStatusIndicator(state.saveStatus),
          const SizedBox(width: 16),
          // 字数统计
          Text(
            '${state.wordCount} 字',
            style: TextStyle(fontSize: 12, color: Colors.grey.shade600),
          ),
          const Spacer(),
          // AI 整理按钮
          IconButton(
            icon: Icon(
              _showAnalysis ? Icons.auto_awesome : Icons.auto_awesome_outlined,
              size: 20,
              color: _showAnalysis ? Colors.indigo : null,
            ),
            onPressed: () {
              setState(() => _showAnalysis = !_showAnalysis);
              // 打开面板时如有缓存分析，直接展示
              final id = context.read<EditorBloc>().state.chapterId;
              if (!_showAnalysis && id != null) {
                context.read<AnalyzeBloc>().add(const ClearAnalysis());
              }
            },
            tooltip: _showAnalysis ? '关闭 AI 整理' : 'AI 整理',
          ),
          // 预览按钮
          IconButton(
            icon: Icon(
              state.isPreviewMode ? Icons.edit : Icons.preview,
              size: 20,
            ),
            onPressed: () {
              // TODO: 切换预览模式
            },
            tooltip: state.isPreviewMode ? '编辑模式' : '预览模式',
          ),
          // 全屏按钮
          IconButton(
            icon: Icon(
              state.isFullScreen ? Icons.fullscreen_exit : Icons.fullscreen,
              size: 20,
            ),
            onPressed: () {
              // TODO: 切换全屏模式
            },
            tooltip: state.isFullScreen ? '退出全屏' : '全屏模式',
          ),
        ],
      ),
    );
  }

  /// 构建保存状态指示器
  Widget _buildSaveStatusIndicator(SaveStatus status) {
    IconData icon;
    Color color;
    String text;

    switch (status) {
      case SaveStatus.saved:
        icon = Icons.check_circle;
        color = Colors.green;
        text = '已保存';
        break;
      case SaveStatus.saving:
        icon = Icons.hourglass_empty;
        color = Colors.orange;
        text = '保存中...';
        break;
      case SaveStatus.unsaved:
        icon = Icons.circle;
        color = Colors.grey;
        text = '未保存';
        break;
      case SaveStatus.error:
        icon = Icons.error;
        color = Colors.red;
        text = '保存失败';
        break;
    }

    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(icon, size: 16, color: color),
        const SizedBox(width: 4),
        Text(
          text,
          style: TextStyle(fontSize: 12, color: color),
        ),
      ],
    );
  }

  /// 构建编辑器内容
  Widget _buildEditorContent(BuildContext context, EditorState state) {
    // 从分析层读取标注（拆分建议 + 场景）——只读 overlay，不进入文本
    return BlocBuilder<AnalyzeBloc, AnalysisState>(
      builder: (context, analysisState) {
        final analysis = analysisState.analysis;
        return Padding(
          padding: const EdgeInsets.all(16),
          child: _EditorArea(
            key: ValueKey(state.chapterId),
            initialContent: state.content,
            splitPoints: analysis?.splitPoints ?? const [],
            scenes: analysis?.scenes ?? const [],
            onChanged: (value) {
              context.read<EditorBloc>().add(UpdateContent(value));
            },
          ),
        );
      },
    );
  }

  /// 构建状态栏
  Widget _buildStatusBar(BuildContext context, EditorState state) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      decoration: BoxDecoration(
        color: Colors.white,
        border: Border(
          top: BorderSide(color: Colors.grey.shade200),
        ),
      ),
      child: Row(
        children: [
          Text(
            '光标位置: ${state.cursorPosition}',
            style: TextStyle(fontSize: 11, color: Colors.grey.shade600),
          ),
          const Spacer(),
          Text(
            'Ctrl+S 保存 | Ctrl+Z 撤销',
            style: TextStyle(fontSize: 11, color: Colors.grey.shade500),
          ),
        ],
      ),
    );
  }
}

/// 编辑器内容区——持有稳定的 TextEditingController + 只读标注层。
///
/// 关键：controller 只在章节切换时重建（ValueKey 驱动），
/// 避免每次 build 新建导致光标丢失、输入中断。
/// 标注层（拆分点虚线/场景色条）叠加在文本之上，点击跳转对应行。
class _EditorArea extends StatefulWidget {
  final String initialContent;
  final List<SplitPoint> splitPoints;
  final List<Scene> scenes;
  final void Function(String) onChanged;

  const _EditorArea({
    super.key,
    required this.initialContent,
    required this.splitPoints,
    required this.scenes,
    required this.onChanged,
  });

  @override
  State<_EditorArea> createState() => _EditorAreaState();
}

class _EditorAreaState extends State<_EditorArea> {
  static const _fontSize = 16.0;
  static const _lineHeight = _fontSize * 1.8; // 与编辑器 style 一致

  late final TextEditingController _controller;
  final ScrollController _scrollController = ScrollController();
  final FocusNode _focusNode = FocusNode();
  double _scrollOffset = 0;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController(text: widget.initialContent);
    _scrollController.addListener(() {
      setState(() => _scrollOffset = _scrollController.offset);
    });
  }

  @override
  void didUpdateWidget(_EditorArea oldWidget) {
    super.didUpdateWidget(oldWidget);
    // key 为 chapterId，章节切换时整个 State 重建；
    // 同章节内容刷新（如保存后）无需重置 controller
  }

  @override
  void dispose() {
    _controller.dispose();
    _scrollController.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  /// 行号 → 字符 offset（按 \n 累计）
  int _offsetOfLine(int line) {
    final lines = _controller.text.split('\n');
    if (line < 1 || line > lines.length) return -1;
    var offset = 0;
    for (var i = 0; i < line - 1; i++) {
      offset += lines[i].length + 1;
    }
    return offset;
  }

  void _jumpToLine(int line) {
    final offset = _offsetOfLine(line);
    if (offset < 0) return;
    // 光标跳转
    _controller.selection = TextSelection.collapsed(offset: offset);
    _focusNode.requestFocus();
    // 滚动到目标行（上方留 60px 余量）
    if (_scrollController.hasClients) {
      final target = ((line - 1) * _lineHeight) - 60;
      final max = _scrollController.position.maxScrollExtent;
      _scrollController.jumpTo(target.clamp(0.0, max));
    }
  }

  void _onSplitTap(SplitPoint sp) {
    _jumpToLine(sp.line);
    ScaffoldMessenger.of(context)
      ..hideCurrentSnackBar()
      ..showSnackBar(SnackBar(
        content: Text('拆分建议 L${sp.line}：${sp.reason}'),
        duration: const Duration(seconds: 3),
      ));
  }

  void _onSceneTap(Scene s) {
    _jumpToLine(s.startLine);
    final chars = s.characters.isEmpty ? '' : '（${s.characters.join('、')}）';
    ScaffoldMessenger.of(context)
      ..hideCurrentSnackBar()
      ..showSnackBar(SnackBar(
        content: Text('场景 L${s.startLine}-L${s.endLine}$chars：${s.summary}'),
        duration: const Duration(seconds: 3),
      ));
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        return Stack(
          children: [
            TextField(
              controller: _controller,
              scrollController: _scrollController,
              focusNode: _focusNode,
              maxLines: null,
              expands: true,
              textAlignVertical: TextAlignVertical.top,
              decoration: const InputDecoration(
                border: InputBorder.none,
                hintText: '开始写作...',
              ),
              style: const TextStyle(
                fontSize: _fontSize,
                height: 1.8,
              ),
              onChanged: widget.onChanged,
            ),
            Positioned.fill(
              child: AnnotationOverlay(
                splitPoints: widget.splitPoints,
                scenes: widget.scenes,
                lineHeight: _lineHeight,
                scrollOffset: _scrollOffset,
                viewportHeight: constraints.maxHeight,
                onSplitTap: _onSplitTap,
                onSceneTap: _onSceneTap,
              ),
            ),
          ],
        );
      },
    );
  }
}
