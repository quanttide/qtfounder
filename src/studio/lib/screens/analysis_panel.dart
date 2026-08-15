/// AI 整理面板——展示分析结果，只读为主，采纳为显式动作。
library;

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../bloc/analyze/analyze_bloc.dart';
import '../models/analysis.dart';

/// 右侧 AI 整理面板
class AnalysisPanel extends StatelessWidget {
  final String? chapterId; // 当前编辑器章节

  const AnalysisPanel({super.key, required this.chapterId});

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 300,
      decoration: const BoxDecoration(
        color: Colors.white,
        border: Border(left: BorderSide(color: Color(0xFFE2E8F0))),
      ),
      child: BlocBuilder<AnalyzeBloc, AnalysisState>(
        builder: (context, state) {
          if (state.isLoading) {
            return const _LoadingView();
          }
          if (state.error != null) {
            return _ErrorView(
              message: state.error!,
              onRetry: () {
                final id = chapterId;
                if (id != null) {
                  context.read<AnalyzeBloc>().add(RefreshAnalysis(id));
                }
              },
            );
          }
          final analysis = state.analysis;
          if (analysis == null || chapterId == null ||
              analysis.chapterId != chapterId) {
            return _EmptyView(
              chapterId: chapterId,
              onAnalyze: () {
                if (chapterId != null) {
                  context.read<AnalyzeBloc>().add(AnalyzeChapter(chapterId!));
                }
              },
            );
          }
          return _AnalysisContent(analysis: analysis);
        },
      ),
    );
  }
}

class _LoadingView extends StatelessWidget {
  const _LoadingView();

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const CircularProgressIndicator(),
          const SizedBox(height: 16),
          const Text(
            '正在分析...',
            style: TextStyle(fontSize: 14, fontWeight: FontWeight.w600),
          ),
          const SizedBox(height: 4),
          Text(
            '首次分析约需 20-40 秒\n（全文发送给 LLM，仅结构输出）',
            textAlign: TextAlign.center,
            style: TextStyle(fontSize: 11, color: Colors.grey.shade600),
          ),
        ],
      ),
    );
  }
}

class _EmptyView extends StatelessWidget {
  final String? chapterId;
  final VoidCallback onAnalyze;

  const _EmptyView({required this.chapterId, required this.onAnalyze});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.auto_awesome, size: 48, color: Colors.grey.shade400),
          const SizedBox(height: 12),
          const Text(
            'AI 整理',
            style: TextStyle(fontSize: 15, fontWeight: FontWeight.w600),
          ),
          const SizedBox(height: 4),
          Text(
            chapterId == null
                ? '选择一个章节后开始分析'
                : '分析文本结构：摘要、标签、场景、归类建议。\n不改写原文。',
            textAlign: TextAlign.center,
            style: TextStyle(fontSize: 12, color: Colors.grey.shade600),
          ),
          const SizedBox(height: 16),
          FilledButton.icon(
            onPressed: chapterId == null ? null : onAnalyze,
            icon: const Icon(Icons.analytics_outlined, size: 16),
            label: const Text('开始分析'),
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
    return Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(Icons.error_outline, size: 40, color: Colors.red.shade300),
          const SizedBox(height: 12),
          Text(
            message,
            textAlign: TextAlign.center,
            style: TextStyle(fontSize: 12, color: Colors.grey.shade700),
          ),
          const SizedBox(height: 12),
          OutlinedButton(onPressed: onRetry, child: const Text('重试')),
        ],
      ),
    );
  }
}

class _AnalysisContent extends StatelessWidget {
  final ChapterAnalysis analysis;

  const _AnalysisContent({required this.analysis});

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Row(
          children: [
            const Text(
              'AI 整理',
              style: TextStyle(fontSize: 14, fontWeight: FontWeight.w700),
            ),
            const Spacer(),
            IconButton(
              icon: const Icon(Icons.refresh, size: 16),
              tooltip: '重新分析',
              onPressed: () {
                context.read<AnalyzeBloc>().add(RefreshAnalysis(analysis.chapterId));
              },
            ),
          ],
        ),
        Text(
          '模型: ${analysis.model} · ${_fmtTime(analysis.analyzedAt)}',
          style: TextStyle(fontSize: 10, color: Colors.grey.shade500),
        ),
        const SizedBox(height: 12),

        // 0_日志：灵感分解（分阶段方法）
        if (analysis.stageId == '0_日志') ...[_buildInspirationSection(context)],

        // 其他阶段：结构分析
        if (analysis.stageId != '0_日志') ...[_buildStructureSection(context)],
      ],
    );
  }

  /// 0_日志：灵感片段区块（采纳 → 生成 1_灵感 文件）
  Widget _buildInspirationSection(BuildContext context) {
    if (analysis.inspirationSplits.isEmpty) {
      return const _Card(child: Text('未发现可分解的灵感片段', style: TextStyle(fontSize: 12)));
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const _SectionTitle('灵感片段（可采纳为 1_灵感）'),
        ...analysis.inspirationSplits.map((split) => _Card(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Expanded(
                        child: Text(
                          split.title,
                          style: const TextStyle(
                              fontSize: 12, fontWeight: FontWeight.w700),
                        ),
                      ),
                      Text(
                        'L${split.startLine}-L${split.endLine}',
                        style: TextStyle(
                          fontSize: 11,
                          fontWeight: FontWeight.w600,
                          color: Colors.indigo.shade600,
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 4),
                  Text(
                    split.summary,
                    style: const TextStyle(fontSize: 12, height: 1.5),
                  ),
                  const SizedBox(height: 6),
                  Align(
                    alignment: Alignment.centerRight,
                    child: FilledButton.tonalIcon(
                      onPressed: () {
                        context
                            .read<AnalyzeBloc>()
                            .add(ApplyInspiration(split));
                      },
                      icon: const Icon(Icons.add, size: 14),
                      label: const Text('采纳为灵感', style: TextStyle(fontSize: 11)),
                      style: FilledButton.styleFrom(
                        visualDensity: VisualDensity.compact,
                      ),
                    ),
                  ),
                ],
              ),
            )),
      ],
    );
  }

  /// 其他阶段：结构分析区块
  Widget _buildStructureSection(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // 阶段归类建议
        if (analysis.suggestedStageId != null) ...[_buildStageSuggestion()],

        // 标签
        const _SectionTitle('标签'),
        Wrap(
          spacing: 6,
          runSpacing: 6,
          children: analysis.tags
              .map((t) => Chip(
                    label: Text(t, style: const TextStyle(fontSize: 11)),
                    visualDensity: VisualDensity.compact,
                    backgroundColor: Colors.indigo.shade50,
                    side: BorderSide.none,
                  ))
              .toList(),
        ),
        const SizedBox(height: 12),

        // 摘要
        const _SectionTitle('摘要'),
        _Card(
          child: Text(
            analysis.summary,
            style: const TextStyle(fontSize: 12, height: 1.6),
          ),
        ),
        const SizedBox(height: 12),

        // 拆分建议
        if (analysis.splitPoints.isNotEmpty) ...[
          const _SectionTitle('拆分建议'),
          ...analysis.splitPoints.map((sp) => _Card(
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'L${sp.line}',
                      style: TextStyle(
                        fontSize: 11,
                        fontWeight: FontWeight.w600,
                        color: Colors.indigo.shade600,
                      ),
                    ),
                    const SizedBox(width: 8),
                    Expanded(
                      child: Text(sp.reason, style: const TextStyle(fontSize: 12)),
                    ),
                  ],
                ),
              )),
          const SizedBox(height: 12),
        ],

        // 场景时间线
        if (analysis.scenes.isNotEmpty) ...[
          const _SectionTitle('场景时间线'),
          ...analysis.scenes.map((s) => _Card(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'L${s.startLine}-L${s.endLine}',
                      style: TextStyle(
                        fontSize: 11,
                        fontWeight: FontWeight.w600,
                        color: Colors.grey.shade700,
                      ),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      s.summary,
                      style: const TextStyle(fontSize: 12, height: 1.5),
                    ),
                    if (s.characters.isNotEmpty) ...[
                      const SizedBox(height: 4),
                      Text(
                        '在场: ${s.characters.join('、')}',
                        style: TextStyle(fontSize: 10, color: Colors.grey.shade500),
                      ),
                    ],
                  ],
                ),
              )),
        ],
      ],
    );
  }

  /// 阶段归类建议卡片
  Widget _buildStageSuggestion() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const _SectionTitle('阶段归类建议'),
        _Card(
          child: Row(
            children: [
              Icon(Icons.flag_outlined, size: 14, color: Colors.indigo.shade400),
              const SizedBox(width: 6),
              Expanded(
                child: Text(
                  '建议归入: ${analysis.suggestedStageId}',
                  style: const TextStyle(fontSize: 12),
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 12),
      ],
    );
  }

  String _fmtTime(DateTime dt) =>
      '${dt.year}-${dt.month.toString().padLeft(2, '0')}-${dt.day.toString().padLeft(2, '0')}';
}

class _SectionTitle extends StatelessWidget {
  final String title;

  const _SectionTitle(this.title);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 6),
      child: Text(
        title,
        style: TextStyle(
          fontSize: 11,
          fontWeight: FontWeight.w700,
          color: Colors.grey.shade600,
        ),
      ),
    );
  }
}

class _Card extends StatelessWidget {
  final Widget child;

  const _Card({required this.child});

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      margin: const EdgeInsets.only(bottom: 6),
      padding: const EdgeInsets.all(10),
      decoration: BoxDecoration(
        color: const Color(0xFFF8FAFC),
        borderRadius: BorderRadius.circular(8),
      ),
      child: child,
    );
  }
}
