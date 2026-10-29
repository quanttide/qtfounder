import 'package:flutter/material.dart';

import 'asset/screens/asset_shell.dart';
import 'data/screens/rules_page.dart';

void main() {
  runApp(const FounderApp());
}

/// 应用入口：导航栏 + 命名路由。
///
/// 业务代码已归档至 `data/archive/platform/qtfounder/studio/`；重建时在
/// `lib/` 下按模块分层添加。当前已重建的是 `asset/`（资产抽查）与
/// `data/`（标注规则设计，任务包给标注端）。
class FounderApp extends StatelessWidget {
  const FounderApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: '量潮创始人工作台',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF4F46E5),
          surface: Colors.white,
          brightness: Brightness.light,
        ),
        scaffoldBackgroundColor: const Color(0xFFF1F5F9),
        useMaterial3: true,
        fontFamily: 'Inter',
      ),
      routes: {
        AppShell.dataRoute: (_) => const AppShell(initialIndex: 0),
        AppShell.assetRoute: (_) => const AppShell(initialIndex: 1),
      },
      home: const AppShell(initialIndex: 0),
    );
  }
}

/// 应用壳：左侧导航栏，右侧内容区。
///
/// 两个页面叠在 [IndexedStack] 里，来回切换不丢批次状态。
class AppShell extends StatefulWidget {
  const AppShell({super.key, this.initialIndex = 0});

  /// 「数据」页路由与导航栏下标。
  static const dataRoute = '/data';

  /// 「资产」页路由与导航栏下标。
  static const assetRoute = '/assets';

  final int initialIndex;

  @override
  State<AppShell> createState() => _AppShellState();
}

class _AppShellState extends State<AppShell> {
  late int _index = widget.initialIndex;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Row(
        children: [
          NavigationRail(
            selectedIndex: _index,
            onDestinationSelected: (index) => setState(() => _index = index),
            labelType: NavigationRailLabelType.all,
            leading: const Padding(
              padding: EdgeInsets.symmetric(vertical: 16),
              child: Icon(Icons.dashboard_outlined, size: 32),
            ),
            destinations: const [
              NavigationRailDestination(
                icon: Icon(Icons.notes_outlined),
                selectedIcon: Icon(Icons.notes),
                label: Text('数据'),
              ),
              NavigationRailDestination(
                icon: Icon(Icons.fact_check_outlined),
                selectedIcon: Icon(Icons.fact_check),
                label: Text('资产'),
              ),
            ],
          ),
          const VerticalDivider(width: 1, thickness: 1),
          Expanded(
            child: IndexedStack(
              index: _index,
              children: const [DataRulesPage(), AssetAuditPage()],
            ),
          ),
        ],
      ),
    );
  }
}
