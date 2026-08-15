/// 资产职能页——契约注册表（列出所有资产契约）
library;

import 'package:flutter/material.dart';

/// 资产注册项
class AssetEntry {
  final String id;
  final String label;
  final String icon;
  final String description;

  const AssetEntry({
    required this.id,
    required this.label,
    required this.icon,
    required this.description,
  });
}

/// 资产注册表（契约列表——与 contracts/ 目录对应）
const kAssetEntries = [
  AssetEntry(
    id: 'fiction',
    label: '小说',
    icon: 'menu_book',
    description: '职场言情 / 校园言情 / 重生言情——创作产出资产',
  ),
  AssetEntry(
    id: 'memory',
    label: '记忆',
    icon: 'memory',
    description: 'context / intention / journal / roadmap——第二大脑资产',
  ),
];

class AssetsScreen extends StatelessWidget {
  final void Function(AssetEntry entry) onOpen;

  const AssetsScreen({super.key, required this.onOpen});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(28, 28, 28, 0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Text(
            '资产',
            style: TextStyle(fontSize: 24, fontWeight: FontWeight.w700, color: Color(0xFF1E293B)),
          ),
          const SizedBox(height: 4),
          const Text(
            '第二大脑资产——按资产契约解读，结构即界面',
            style: TextStyle(fontSize: 13, color: Color(0xFF94A3B8)),
          ),
          const SizedBox(height: 20),
          Expanded(
            child: ListView(
              children: kAssetEntries.map((entry) {
                return Padding(
                  padding: const EdgeInsets.only(bottom: 12),
                  child: _AssetCard(entry: entry, onTap: () => onOpen(entry)),
                );
              }).toList(),
            ),
          ),
        ],
      ),
    );
  }
}

class _AssetCard extends StatelessWidget {
  final AssetEntry entry;
  final VoidCallback onTap;

  const _AssetCard({required this.entry, required this.onTap});

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(12),
      child: Container(
        padding: const EdgeInsets.all(16),
        decoration: BoxDecoration(
          color: Colors.white,
          borderRadius: BorderRadius.circular(12),
        ),
        child: Row(
          children: [
            Container(
              width: 40,
              height: 40,
              decoration: BoxDecoration(
                color: const Color(0xFFE0E7FF),
                borderRadius: BorderRadius.circular(10),
              ),
              child: Icon(
                entry.icon == 'menu_book' ? Icons.menu_book_outlined : Icons.memory_outlined,
                size: 20,
                color: const Color(0xFF4F46E5),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    entry.label,
                    style: const TextStyle(fontSize: 15, fontWeight: FontWeight.w700, color: Color(0xFF1E293B)),
                  ),
                  const SizedBox(height: 2),
                  Text(
                    entry.description,
                    style: const TextStyle(fontSize: 12, color: Color(0xFF64748B)),
                  ),
                ],
              ),
            ),
            const Icon(Icons.chevron_right, size: 18, color: Color(0xFF94A3B8)),
          ],
        ),
      ),
    );
  }
}
