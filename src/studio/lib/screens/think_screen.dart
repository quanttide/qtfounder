/// 思考职能页——情绪结构化处理器（思考云原型）。
///
/// 左侧：日志列表（按日期倒序，处理状态标记）。
/// 右侧：结构化工作台——日志原文 + AI 四分类结果（事实/感受/需要/行动），
/// 条目可忽略、可查看原文引用，采纳后沉淀为日志 `## 结构化（思考云）` 段。
library;

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../bloc/think/think_bloc.dart';
import '../models/emotion.dart';
import '../repositories/emotion_repository.dart';

class ThinkScreen extends StatefulWidget {
  const ThinkScreen({super.key});

  @override
  State<ThinkScreen> createState() => _ThinkScreenState();
}

class _ThinkScreenState extends State<ThinkScreen> {
  bool _showJournal = false;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Row(
        children: [
          // 左侧：日志列表
          SizedBox(width: 280, child: _buildJournalList(context)),
          // 右侧：结构化工作台
          Expanded(child: _buildWorkspace(context)),
        ],
      ),
    );
  }

  /// 日志列表（左侧）
  Widget _buildJournalList(BuildContext context) {
    return Container(
      color: Colors.white,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Padding(
            padding: EdgeInsets.fromLTRB(16, 16, 16, 8),
            child: Text(
              '每日日志',
              style: TextStyle(fontSize: 13, fontWeight: FontWeight.w700, color: Color(0xFF1E293B)),
            ),
          ),
          const Padding(
            padding: EdgeInsets.symmetric(horizontal: 16),
            child: Text(
              '日志是输入，结构化是输出',
              style: TextStyle(fontSize: 11, color: Color(0xFF94A3B8)),
            ),
          ),
          const SizedBox(height: 8),
          Expanded(
            child: BlocBuilder<ThinkBloc, ThinkState>(
              builder: (context, state) {
                if (state.journals.isEmpty) {
                  return const Padding(
                    padding: EdgeInsets.all(16),
                    child: Text('暂无日志', style: TextStyle(fontSize: 12, color: Color(0xFF94A3B8))),
                  );
                }
                return ListView.builder(
                  itemCount: state.journals.length,
                  itemBuilder: (context, index) {
                    final journal = state.journals[index];
                    final selected = journal.date == state.selectedDate;
                    return _buildJournalTile(context, journal, selected);
                  },
                );
              },
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildJournalTile(BuildContext context, JournalEntry journal, bool selected) {
    return ListTile(
      dense: true,
      selected: selected,
      selectedTileColor: Colors.indigo.shade50,
      leading: Icon(
        journal.processed ? Icons.check_circle_outline : Icons.circle_outlined,
        size: 16,
        color: journal.processed ? Colors.green.shade600 : Colors.grey.shade400,
      ),
      title: Text(
        journal.date,
        style: TextStyle(
          fontSize: 13,
          fontWeight: selected ? FontWeight.w600 : FontWeight.w400,
        ),
      ),
      subtitle: Text(
        journal.processed ? '已结构化' : '未处理',
        style: TextStyle(fontSize: 10, color: Colors.grey.shade500),
      ),
      onTap: () => context.read<ThinkBloc>().add(SelectJournal(journal.date)),
    );
  }

  /// 结构化工作台（右侧）
  Widget _buildWorkspace(BuildContext context) {
    return BlocBuilder<ThinkBloc, ThinkState>(
      builder: (context, state) {
        if (state.error != null) {
          return _ErrorView(
            message: state.error!,
            onRetry: () => context.read<ThinkBloc>().add(const LoadJournals()),
          );
        }
        if (state.selectedDate == null) {
          return const _EmptyView();
        }
        return _buildJournalWorkspace(context, state);
      },
    );
  }

  Widget _buildJournalWorkspace(BuildContext context, ThinkState state) {
    final content = state.journalContent ?? '';
    final analysis = state.analysis;
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 顶部：日期 + 操作
          Row(
            children: [
              Text(
                state.selectedDate!,
                style: const TextStyle(fontSize: 20, fontWeight: FontWeight.w700, color: Color(0xFF1E293B)),
              ),
              const SizedBox(width: 12),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                decoration: BoxDecoration(
                  color: analysis != null ? Colors.green.shade50 : Colors.grey.shade200,
                  borderRadius: BorderRadius.circular(10),
                ),
                child: Text(
                  analysis != null ? '已整理' : '未整理',
                  style: TextStyle(
                    fontSize: 11,
                    fontWeight: FontWeight.w600,
                    color: analysis != null ? Colors.green.shade700 : Colors.grey.shade600,
                  ),
                ),
              ),
              const Spacer(),
              if (analysis != null) ...[
                OutlinedButton.icon(
                  onPressed: state.isAdopting
                      ? null
                      : () => context.read<ThinkBloc>().add(const AnalyzeJournal(force: true)),
                  icon: const Icon(Icons.refresh, size: 14),
                  label: const Text('重新整理', style: TextStyle(fontSize: 12)),
                  style: OutlinedButton.styleFrom(visualDensity: VisualDensity.compact),
                ),
                const SizedBox(width: 8),
                FilledButton.icon(
                  onPressed: state.isAdopting
                      ? null
                      : () => context.read<ThinkBloc>().add(const AdoptStructure()),
                  icon: state.isAdopting
                      ? const SizedBox(
                          width: 14,
                          height: 14,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.check, size: 14),
                  label: Text(state.isAdopting ? '采纳中...' : '采纳到日志', style: const TextStyle(fontSize: 12)),
                  style: FilledButton.styleFrom(visualDensity: VisualDensity.compact),
                ),
              ] else ...[
                FilledButton.icon(
                  onPressed: state.isLoading
                      ? null
                      : () => context.read<ThinkBloc>().add(const AnalyzeJournal()),
                  icon: state.isLoading
                      ? const SizedBox(
                          width: 14,
                          height: 14,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.auto_awesome, size: 14),
                  label: Text(state.isLoading ? '整理中...' : 'AI 整理', style: const TextStyle(fontSize: 12)),
                  style: FilledButton.styleFrom(visualDensity: VisualDensity.compact),
                ),
              ],
            ],
          ),
          const SizedBox(height: 4),
          Text(
            '只分类，不替你想——把混沌拆成 事实/感受/需要/行动',
            style: TextStyle(fontSize: 11, color: Colors.grey.shade500),
          ),
          const SizedBox(height: 16),
          Expanded(
            child: state.isLoading
                ? const _LoadingView()
                : ListView(
                    children: [
                      if (analysis != null) _buildAnalysisSections(context, analysis, content),
                      _buildJournalSection(context, content),
                    ],
                  ),
          ),
        ],
      ),
    );
  }

  /// 四分类结果区
  Widget _buildAnalysisSections(BuildContext context, EmotionAnalysis analysis, String content) {
    final sections = <Widget>[];
    for (final category in EmotionCategory.values) {
      final entries = analysis.byCategory(category);
      sections.add(_buildCategorySection(context, category, entries, content));
    }
    sections.add(const SizedBox(height: 16));
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const _SectionTitle('结构化结果（思考云）'),
        ...sections,
      ],
    );
  }

  Widget _buildCategorySection(
    BuildContext context,
    EmotionCategory category,
    List<EmotionEntry> entries,
    String content,
  ) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Text(
                category.label,
                style: const TextStyle(fontSize: 13, fontWeight: FontWeight.w700, color: Color(0xFF334155)),
              ),
              const SizedBox(width: 6),
              Text(
                category.hint,
                style: TextStyle(fontSize: 11, color: Colors.grey.shade500),
              ),
              const SizedBox(width: 6),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 1),
                decoration: BoxDecoration(
                  color: Colors.grey.shade200,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Text('${entries.length}', style: const TextStyle(fontSize: 10)),
              ),
            ],
          ),
          const SizedBox(height: 4),
          if (entries.isEmpty)
            Text('（无）', style: TextStyle(fontSize: 12, color: Colors.grey.shade400))
          else
            ...entries.map((e) => _buildEntryCard(context, e, content)),
        ],
      ),
    );
  }

  /// 单条条目：提炼 + 原文引用 + 忽略
  Widget _buildEntryCard(BuildContext context, EmotionEntry entry, String content) {
    final excerpt = _excerptOf(content, entry.startLine, entry.endLine);
    return Container(
      margin: const EdgeInsets.only(bottom: 6),
      padding: const EdgeInsets.all(10),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.grey.shade200),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(entry.text, style: const TextStyle(fontSize: 12.5, height: 1.5)),
                if (excerpt.isNotEmpty) ...[
                  const SizedBox(height: 4),
                  Text(
                    'L${entry.startLine}-L${entry.endLine} · $excerpt',
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(fontSize: 11, color: Colors.grey.shade500, height: 1.4),
                  ),
                ],
              ],
            ),
          ),
          IconButton(
            tooltip: '忽略此条',
            onPressed: () => context.read<ThinkBloc>().add(DismissEntry(entry)),
            icon: const Icon(Icons.close, size: 14),
            color: Colors.grey.shade400,
            visualDensity: VisualDensity.compact,
            padding: EdgeInsets.zero,
            constraints: const BoxConstraints(minWidth: 24, minHeight: 24),
          ),
        ],
      ),
    );
  }

  /// 原文区（可折叠）
  Widget _buildJournalSection(BuildContext context, String content) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        InkWell(
          onTap: () => setState(() => _showJournal = !_showJournal),
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 4),
            child: Row(
              children: [
                Icon(
                  _showJournal ? Icons.expand_less : Icons.expand_more,
                  size: 16,
                  color: Colors.grey.shade600,
                ),
                const SizedBox(width: 4),
                Text(
                  '日志原文（${_lineCount(content)} 行）',
                  style: const TextStyle(fontSize: 13, fontWeight: FontWeight.w700, color: Color(0xFF334155)),
                ),
              ],
            ),
          ),
        ),
        if (_showJournal)
          Container(
            width: double.infinity,
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: const Color(0xFFF8FAFC),
              borderRadius: BorderRadius.circular(8),
              border: Border.all(color: Colors.grey.shade200),
            ),
            child: Text(
              content,
              style: const TextStyle(fontSize: 12, height: 1.7, color: Color(0xFF475569)),
            ),
          ),
      ],
    );
  }

  /// 按行号范围截取原文（用于引用展示）
  String _excerptOf(String content, int startLine, int endLine) {
    final lines = content.split('\n');
    final start = startLine.clamp(1, lines.length);
    final end = endLine.clamp(start, lines.length);
    return lines.sublist(start - 1, end).join(' ').replaceAll(RegExp(r'\s+'), ' ').trim();
  }

  int _lineCount(String content) => content.split('\n').length;
}

class _SectionTitle extends StatelessWidget {
  final String text;

  const _SectionTitle(this.text);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Text(
        text,
        style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w700, color: Color(0xFF1E293B)),
      ),
    );
  }
}

class _LoadingView extends StatelessWidget {
  const _LoadingView();

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(32),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const CircularProgressIndicator(),
          const SizedBox(height: 16),
          const Text(
            '正在整理...',
            style: TextStyle(fontSize: 14, fontWeight: FontWeight.w600),
          ),
          const SizedBox(height: 4),
          Text(
            '四分类（事实/感受/需要/行动），约需 20-40 秒',
            textAlign: TextAlign.center,
            style: TextStyle(fontSize: 11, color: Colors.grey.shade600),
          ),
        ],
      ),
    );
  }
}

class _EmptyView extends StatelessWidget {
  const _EmptyView();

  @override
  Widget build(BuildContext context) {
    return const Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.insights_outlined, size: 40, color: Color(0xFFCBD5E1)),
          SizedBox(height: 12),
          Text(
            '选择一天日志开始整理',
            style: TextStyle(fontSize: 14, fontWeight: FontWeight.w600, color: Color(0xFF64748B)),
          ),
          SizedBox(height: 4),
          Text(
            '把混沌拆成 事实/感受/需要/行动——不替你想，只帮你摆出来',
            style: TextStyle(fontSize: 11, color: Color(0xFF94A3B8)),
          ),
        ],
      ),
    );
  }
}

class _ErrorView extends StatelessWidget {
  final String message;
  final VoidCallback onRetry;

  const _ErrorView({required this.message, required this.onRetry});

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Text(message, style: const TextStyle(fontSize: 12, color: Color(0xFFB91C1C))),
          const SizedBox(height: 8),
          OutlinedButton(onPressed: onRetry, child: const Text('重试', style: TextStyle(fontSize: 12))),
        ],
      ),
    );
  }
}
