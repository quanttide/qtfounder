import 'package:flutter/material.dart';

import 'config.dart';
import 'data/creative_repository.dart';

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
                    '量潮',
                    style: TextStyle(
                      fontSize: 16,
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

/// 创作现场——工作台主界面（数据源：fiction + memory）
class CreativeDesk extends StatefulWidget {
  /// 测试/定制用 loader 注入（默认读真实数据源）
  final Future<List<CreativeItem>> Function()? chaptersLoader;
  final Future<List<CreativeItem>> Function()? memoryLoader;

  const CreativeDesk({super.key, this.chaptersLoader, this.memoryLoader});

  @override
  State<CreativeDesk> createState() => _CreativeDeskState();
}

class _CreativeDeskState extends State<CreativeDesk> {
  List<CreativeItem> _chapters = [];
  List<CreativeItem> _memoryDocs = [];
  bool _loading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final chapters =
          await (widget.chaptersLoader ?? loadRevisionChapters)();
      final docs = await (widget.memoryLoader ?? loadMemoryDocs)();
      setState(() {
        _chapters = chapters;
        _memoryDocs = docs;
        _loading = false;
      });
    } catch (e) {
      setState(() {
        _error = '数据源加载失败：$e';
        _loading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Center(child: CircularProgressIndicator());
    }
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
          Text(
            dataSourceAvailable
                ? '数据源：$fictionPath\n          $memoryPath'
                : '数据源：fiction + memory（内置示例，Web 无文件系统）',
            style: const TextStyle(fontSize: 12, color: Color(0xFF94A3B8)),
          ),
          const SizedBox(height: 20),
          Expanded(
            child: ListView(
              children: [
                if (_error != null)
                  Padding(
                    padding: const EdgeInsets.only(bottom: 12),
                    child: Text(
                      _error!,
                      style: const TextStyle(color: Color(0xFFEF4444)),
                    ),
                  ),
                _SectionCard(
                  icon: Icons.timeline,
                  title: '改稿轨迹（fiction · 4_改稿）',
                  items: _chapters.map((c) => c.name).toList(),
                ),
                const SizedBox(height: 12),
                _SectionCard(
                  icon: Icons.grid_view_outlined,
                  title: '创作域矩阵（memory）',
                  items: _memoryDocs
                      .map((d) => '[${d.category}] ${d.name}')
                      .toList(),
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
              Expanded(
                child: Text(
                  title,
                  style: const TextStyle(
                    fontSize: 14,
                    fontWeight: FontWeight.w700,
                    color: Color(0xFF1E293B),
                  ),
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
