/// 通用资产目录页——按契约渲染目录树（结构即界面）
library;

import 'package:flutter/material.dart';

import '../data/asset_catalog_engine.dart';
import '../models/asset_catalog.dart';

class AssetCatalogScreen extends StatefulWidget {
  final String assetId;
  final String label;
  final String contractPath; // assets/contracts/xxx.yaml
  final String? dataSourceRoot; // 测试/定制注入

  const AssetCatalogScreen({
    super.key,
    required this.assetId,
    required this.label,
    required this.contractPath,
    this.dataSourceRoot,
  });

  @override
  State<AssetCatalogScreen> createState() => _AssetCatalogScreenState();
}

class _AssetCatalogScreenState extends State<AssetCatalogScreen> {
  AssetCatalog? _catalog;
  String? _error;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final engine = AssetCatalogEngine(
        contractAssetPath: widget.contractPath,
        dataSourceRoot: widget.dataSourceRoot,
      );
      final catalog = await engine.load();
      setState(() => _catalog = catalog);
    } catch (e) {
      setState(() => _error = '资产加载失败：$e');
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_error != null) {
      return Padding(
        padding: const EdgeInsets.all(28),
        child: Text(_error!, style: const TextStyle(color: Color(0xFFEF4444))),
      );
    }
    if (_catalog == null) {
      return const Center(child: CircularProgressIndicator());
    }
    final catalog = _catalog!;
    return Padding(
      padding: const EdgeInsets.fromLTRB(28, 28, 28, 0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            '${catalog.contract.label}资产',
            style: const TextStyle(fontSize: 24, fontWeight: FontWeight.w700, color: Color(0xFF1E293B)),
          ),
          const SizedBox(height: 4),
          Text(
            catalog.nodes.isEmpty
                ? '数据源不可用（Web 无文件系统）'
                : '${catalog.contract.asset} · ${catalog.nodes.length} 个一级目录',
            style: const TextStyle(fontSize: 13, color: Color(0xFF94A3B8)),
          ),
          const SizedBox(height: 20),
          Expanded(
            child: ListView(
              children: catalog.nodes
                  .map((node) => _NodeTile(
                        node: node,
                        onOpenFile: (file) => _openFile(file),
                      ))
                  .toList(),
            ),
          ),
        ],
      ),
    );
  }

  void _openFile(CatalogFile file) {
    // 阅读详情（只读）——显示文件名提示；正文阅读后续实现
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('${file.title}（${file.path}）')),
    );
  }
}

/// 目录节点卡片（可展开）
class _NodeTile extends StatefulWidget {
  final CatalogNode node;
  final void Function(CatalogFile file) onOpenFile;

  const _NodeTile({required this.node, required this.onOpenFile});

  @override
  State<_NodeTile> createState() => _NodeTileState();
}

class _NodeTileState extends State<_NodeTile> {
  bool _expanded = false;

  @override
  Widget build(BuildContext context) {
    final node = widget.node;
    final hasChildren = node.children.isNotEmpty;
    final fileCount = node.files.length +
        node.children.fold<int>(0, (sum, c) => sum + c.files.length);
    return Container(
      margin: const EdgeInsets.only(bottom: 10),
      decoration: BoxDecoration(
        color: Colors.white,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        children: [
          InkWell(
            onTap: () => setState(() => _expanded = !_expanded),
            borderRadius: BorderRadius.circular(12),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
              child: Row(
                children: [
                  Icon(
                    _expanded ? Icons.folder_open : Icons.folder_outlined,
                    size: 18,
                    color: const Color(0xFF4F46E5),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      node.name,
                      style: const TextStyle(fontSize: 14, fontWeight: FontWeight.w600, color: Color(0xFF1E293B)),
                    ),
                  ),
                  Text(
                    '${node.label} · $fileCount',
                    style: const TextStyle(fontSize: 11, color: Color(0xFF94A3B8)),
                  ),
                  Icon(
                    _expanded ? Icons.expand_less : Icons.expand_more,
                    size: 18,
                    color: const Color(0xFF94A3B8),
                  ),
                ],
              ),
            ),
          ),
          if (_expanded && hasChildren)
            ...node.children.map((c) => Padding(
                  padding: const EdgeInsets.only(left: 16),
                  child: _NodeTile(node: c, onOpenFile: widget.onOpenFile),
                )),
          if (_expanded && node.files.isNotEmpty)
            Padding(
              padding: const EdgeInsets.only(left: 16, bottom: 8),
              child: Column(
                children: node.files
                    .map((f) => _FileTile(file: f, onTap: () => widget.onOpenFile(f)))
                    .toList(),
              ),
            ),
        ],
      ),
    );
  }
}

/// 文件条目
class _FileTile extends StatelessWidget {
  final CatalogFile file;
  final VoidCallback onTap;

  const _FileTile({required this.file, required this.onTap});

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(8),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
        child: Row(
          children: [
            const Icon(Icons.description_outlined, size: 15, color: Color(0xFF64748B)),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                file.title,
                style: const TextStyle(fontSize: 13, color: Color(0xFF475569)),
              ),
            ),
            if (file.version > 1)
              Text(
                'v${file.version}',
                style: const TextStyle(fontSize: 10, color: Color(0xFF94A3B8)),
              ),
          ],
        ),
      ),
    );
  }
}
