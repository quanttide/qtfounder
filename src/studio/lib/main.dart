import 'package:flutter/material.dart';

import 'bloc/app_bloc_provider.dart';
import 'screens/asset_catalog_screen.dart';
import 'screens/assets_screen.dart';
import 'screens/create_screen_new.dart';

void main() {
  runApp(const FounderApp());
}

class FounderApp extends StatelessWidget {
  const FounderApp({super.key});

  @override
  Widget build(BuildContext context) {
    return AppBlocProvider(
      child: MaterialApp(
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
        home: const Shell(),
      ),
    );
  }
}

/// 应用壳：品牌侧边栏（量潮 logo + 职能导航）+ 内容区
class Shell extends StatefulWidget {
  const Shell({super.key});

  @override
  State<Shell> createState() => _ShellState();
}

/// 职能导航项
class _NavItem {
  final IconData icon;
  final String label;
  final Widget Function() buildPage;

  const _NavItem({required this.icon, required this.label, required this.buildPage});
}

class _ShellState extends State<Shell> {
  int _selectedIndex = 0;
  late final List<_NavItem> _navItems = [
    _NavItem(
      icon: Icons.folder_outlined,
      label: '资产',
      buildPage: () => AssetsScreen(onOpen: _openAsset),
    ),
    _NavItem(
      icon: Icons.edit_outlined,
      label: '写作',
      buildPage: () => const CreateScreenNew(),
    ),
    _NavItem(
      icon: Icons.insights_outlined,
      label: '思考',
      buildPage: () => const _PlaceholderPage(title: '思考', subtitle: '思绪结构化（思考云原型，规划中）'),
    ),
  ];

  void _openAsset(AssetEntry entry) {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => AssetCatalogScreen(
          assetId: entry.id,
          label: entry.label,
          contractPath: 'assets/contracts/${entry.id}.yaml',
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFFF1F5F9),
      body: SafeArea(
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Container(
              width: 80,
              color: Colors.white,
              child: Column(
                children: [
                  const SizedBox(height: 20),
                  const Text(
                    '量潮',
                    style: TextStyle(
                      fontSize: 16,
                      fontWeight: FontWeight.w700,
                      color: Color(0xFF4F46E5),
                    ),
                  ),
                  const SizedBox(height: 16),
                  ..._navItems.asMap().entries.map((entry) {
                    final i = entry.key;
                    final item = entry.value;
                    final selected = i == _selectedIndex;
                    return Padding(
                      padding: const EdgeInsets.only(bottom: 8),
                      child: Tooltip(
                        message: item.label,
                        child: InkWell(
                          onTap: () => setState(() => _selectedIndex = i),
                          borderRadius: BorderRadius.circular(10),
                          child: Container(
                            width: 40,
                            height: 40,
                            decoration: BoxDecoration(
                              color: selected ? const Color(0xFFE0E7FF) : Colors.transparent,
                              borderRadius: BorderRadius.circular(10),
                            ),
                            child: Icon(
                              item.icon,
                              size: 20,
                              color: selected ? const Color(0xFF4F46E5) : const Color(0xFF94A3B8),
                            ),
                          ),
                        ),
                      ),
                    );
                  }),
                  const Spacer(),
                  const SizedBox(height: 20),
                ],
              ),
            ),
            Expanded(child: _navItems[_selectedIndex].buildPage()),
          ],
        ),
      ),
    );
  }
}

/// 占位页（创作/情绪等未实现职能）
class _PlaceholderPage extends StatelessWidget {
  final String title;
  final String subtitle;

  const _PlaceholderPage({required this.title, required this.subtitle});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(28),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            title,
            style: const TextStyle(fontSize: 24, fontWeight: FontWeight.w700, color: Color(0xFF1E293B)),
          ),
          const SizedBox(height: 4),
          Text(subtitle, style: const TextStyle(fontSize: 13, color: Color(0xFF94A3B8))),
        ],
      ),
    );
  }
}
