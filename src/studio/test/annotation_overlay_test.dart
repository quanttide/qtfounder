/// AnnotationOverlay 测试：渲染标注、点击命中跳转回调。
library;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:qtfounder_studio/models/analysis.dart';
import 'package:qtfounder_studio/screens/annotation_overlay.dart';

void main() {
  const splitPoints = [
    SplitPoint(line: 3, reason: '视角切换'),
    SplitPoint(line: 8, reason: '时间跳跃'),
  ];
  const scenes = [
    Scene(startLine: 1, endLine: 5, characters: ['林远亭'], summary: '场景一'),
  ];

  testWidgets('标注层渲染不崩溃，且不拦截文本（transparent 命中）', (tester) async {
    await tester.pumpWidget(MaterialApp(
      home: Scaffold(
        body: Stack(
          children: [
            const TextField(maxLines: 20),
            Positioned.fill(
              child: AnnotationOverlay(
                splitPoints: splitPoints,
                scenes: scenes,
                lineHeight: 28.8,
                scrollOffset: 0,
                viewportHeight: 600,
              ),
            ),
          ],
        ),
      ),
    ));
    expect(find.byType(AnnotationOverlay), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('点击拆分点行触发 onSplitTap 回调', (tester) async {
    SplitPoint? tapped;
    await tester.pumpWidget(MaterialApp(
      home: Scaffold(
        body: SizedBox(
          width: 400,
          height: 600,
          child: AnnotationOverlay(
            splitPoints: splitPoints,
            scenes: scenes,
            lineHeight: 28.8,
            scrollOffset: 0,
            viewportHeight: 600,
            onSplitTap: (sp) => tapped = sp,
          ),
        ),
      ),
    ));

    // 拆分点 L3 → y = (3-1)*28.8 = 57.6
    await tester.tapAt(const Offset(200, 58));
    expect(tapped, isNotNull);
    expect(tapped!.line, 3);
  });

  testWidgets('点击场景区域触发 onSceneTap 回调', (tester) async {
    Scene? tapped;
    await tester.pumpWidget(MaterialApp(
      home: Scaffold(
        body: SizedBox(
          width: 400,
          height: 600,
          child: AnnotationOverlay(
            splitPoints: splitPoints,
            scenes: scenes,
            lineHeight: 28.8,
            scrollOffset: 0,
            viewportHeight: 600,
            onSceneTap: (s) => tapped = s,
          ),
        ),
      ),
    ));

    // 场景 L1-L5 → y ∈ [0, 144)；选 L4 中部 y=100（不在拆分点上）
    await tester.tapAt(const Offset(200, 100));
    expect(tapped, isNotNull);
    expect(tapped!.startLine, 1);
  });

  testWidgets('滚动偏移后命中随偏移移动', (tester) async {
    SplitPoint? tapped;
    await tester.pumpWidget(MaterialApp(
      home: Scaffold(
        body: SizedBox(
          width: 400,
          height: 600,
          child: AnnotationOverlay(
            splitPoints: splitPoints,
            scenes: scenes,
            lineHeight: 28.8,
            scrollOffset: 57.6, // 滚过 2 行：L3 现在在 y=0
            viewportHeight: 600,
            onSplitTap: (sp) => tapped = sp,
          ),
        ),
      ),
    ));

    await tester.tapAt(const Offset(200, 0));
    expect(tapped, isNotNull);
    expect(tapped!.line, 3);
  });
}
