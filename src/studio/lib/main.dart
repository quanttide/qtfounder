import 'package:flutter/material.dart';

void main() {
  runApp(const FounderApp());
}

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
      home: const Shell(),
    );
  }
}

/// 应用壳：品牌侧边栏 + 内容区
class Shell extends StatelessWidget {
  const Shell({super.key});

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
                    '量',
                    style: TextStyle(
                      fontSize: 20,
                      fontWeight: FontWeight.w700,
                      color: Color(0xFF4F46E5),
                    ),
                  ),
                  const SizedBox(height: 16),
                  Container(
                    width: 40,
                    height: 40,
                    decoration: BoxDecoration(
                      color: const Color(0xFFE0E7FF),
                      borderRadius: BorderRadius.circular(10),
                    ),
                    child: const Icon(
                      Icons.auto_stories_outlined,
                      size: 20,
                      color: Color(0xFF4F46E5),
                    ),
                  ),
                  const Spacer(),
                  const SizedBox(height: 20),
                ],
              ),
            ),
            const Expanded(child: CreativeDesk()),
          ],
        ),
      ),
    );
  }
}

/// 创作现场——工作台主界面
class CreativeDesk extends StatelessWidget {
  const CreativeDesk({super.key});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(28, 28, 28, 0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            '量潮创始人工作台',
            style: TextStyle(
              fontSize: 24,
              fontWeight: FontWeight.w700,
              color: Color(0xFF1E293B),
            ),
          ),
          const SizedBox(height: 4),
          const Text(
            '创作现场——创作流时间线 · 创作域矩阵 · 改稿轨迹',
            style: TextStyle(fontSize: 13, color: Color(0xFF94A3B8)),
          ),
          const SizedBox(height: 20),
          Expanded(
            child: ListView(
              children: const [
                _SectionCard(
                  icon: Icons.timeline,
                  title: '创作流时间线',
                  items: ['小说创作（职场言情 11_x 进行中）', '创始人官网（创作现场转向）', 'qtfounder CLI（health 情绪分析）'],
                ),
                SizedBox(height: 12),
                _SectionCard(
                  icon: Icons.grid_view_outlined,
                  title: '创作域矩阵',
                  items: ['fiction：职场言情 / 校园言情 / 重生言情', 'site：官网（作品集 → 创作现场）', 'cli：health check / track / history / profile'],
                ),
                SizedBox(height: 12),
                _SectionCard(
                  icon: Icons.difference_outlined,
                  title: '改稿轨迹',
                  items: ['1_1 咖啡厅重逢（成稿）', '2_2 男主演讲（脚本待改稿）', '11_x 世界扩张（规划中）'],
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// 分区卡片
class _SectionCard extends StatelessWidget {
  final IconData icon;
  final String title;
  final List<String> items;

  const _SectionCard({
    required this.icon,
    required this.title,
    required this.items,
  });

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(icon, size: 16, color: const Color(0xFF4F46E5)),
              const SizedBox(width: 6),
              Text(
                title,
                style: const TextStyle(
                  fontSize: 14,
                  fontWeight: FontWeight.w700,
                  color: Color(0xFF1E293B),
                ),
              ),
            ],
          ),
          const SizedBox(height: 10),
          ...items.map(
            (item) => Padding(
              padding: const EdgeInsets.only(bottom: 6),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text('· ', style: TextStyle(color: Color(0xFF4F46E5))),
                  Expanded(
                    child: Text(
                      item,
                      style: const TextStyle(fontSize: 12, color: Color(0xFF475569)),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
