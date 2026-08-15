/// 编辑器只读标注层——拆分建议虚线 + 场景色条。
///
/// 渲染在文本层之上（CustomPaint），不进入文本缓冲区——
/// 与原"不改写原文"原则一致：标注是视图，不是内容。
library;

import 'package:flutter/material.dart';

import '../models/analysis.dart';

/// 编辑器标注层
class AnnotationOverlay extends StatelessWidget {
  final List<SplitPoint> splitPoints;
  final List<Scene> scenes;
  final double lineHeight;
  final double scrollOffset;
  final double viewportHeight;
  final void Function(SplitPoint splitPoint)? onSplitTap;
  final void Function(Scene scene)? onSceneTap;

  const AnnotationOverlay({
    super.key,
    required this.splitPoints,
    required this.scenes,
    required this.lineHeight,
    required this.scrollOffset,
    required this.viewportHeight,
    this.onSplitTap,
    this.onSceneTap,
  });

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      behavior: HitTestBehavior.translucent,
      onTapUp: (details) => _hitTest(details.localPosition),
      child: CustomPaint(
        painter: _AnnotationPainter(
          splitPoints: splitPoints,
          scenes: scenes,
          lineHeight: lineHeight,
          scrollOffset: scrollOffset,
          viewportHeight: viewportHeight,
        ),
        child: const SizedBox.expand(),
      ),
    );
  }

  /// 命中检测：拆分点优先（更细粒度），其次场景
  void _hitTest(Offset position) {
    for (final sp in splitPoints) {
      final top = (sp.line - 1) * lineHeight - scrollOffset;
      if (position.dy >= top && position.dy < top + lineHeight) {
        onSplitTap?.call(sp);
        return;
      }
    }
    for (final s in scenes) {
      final top = (s.startLine - 1) * lineHeight - scrollOffset;
      final height = (s.endLine - s.startLine + 1) * lineHeight;
      if (position.dy >= top && position.dy < top + height) {
        onSceneTap?.call(s);
        return;
      }
    }
  }
}

class _AnnotationPainter extends CustomPainter {
  final List<SplitPoint> splitPoints;
  final List<Scene> scenes;
  final double lineHeight;
  final double scrollOffset;
  final double viewportHeight;

  _AnnotationPainter({
    required this.splitPoints,
    required this.scenes,
    required this.lineHeight,
    required this.scrollOffset,
    required this.viewportHeight,
  });

  static const _splitColor = Color(0xFF4F46E5); // indigo
  static const _sceneColor = Color(0x4D4F46E5); // indigo 30%

  @override
  void paint(Canvas canvas, Size size) {
    final splitPaint = Paint()
      ..color = _splitColor
      ..strokeWidth = 1.2;
    final scenePaint = Paint()..color = _sceneColor;

    // 场景色条（左侧 3px）
    for (final s in scenes) {
      final top = (s.startLine - 1) * lineHeight - scrollOffset;
      final height = (s.endLine - s.startLine + 1) * lineHeight;
      if (top + height < 0 || top > viewportHeight) continue; // 可视过滤
      canvas.drawRect(
        Rect.fromLTWH(0, top.clamp(0.0, viewportHeight), 3,
            (top + height).clamp(0.0, viewportHeight) - top.clamp(0.0, viewportHeight)),
        scenePaint,
      );
    }

    // 拆分点虚线（全宽横向线 + 左侧圆点）
    for (final sp in splitPoints) {
      final y = (sp.line - 1) * lineHeight - scrollOffset;
      if (y < 0 || y > viewportHeight) continue; // 可视过滤
      canvas.drawCircle(Offset(8, y), 3, Paint()..color = _splitColor);
      canvas.drawLine(
        Offset(16, y),
        Offset(size.width, y),
        splitPaint,
      );
    }
  }

  @override
  bool shouldRepaint(_AnnotationPainter oldDelegate) =>
      oldDelegate.splitPoints != splitPoints ||
      oldDelegate.scenes != scenes ||
      oldDelegate.lineHeight != lineHeight ||
      oldDelegate.scrollOffset != scrollOffset ||
      oldDelegate.viewportHeight != viewportHeight;
}
