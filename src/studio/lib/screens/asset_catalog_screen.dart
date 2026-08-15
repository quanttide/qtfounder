/// 通用资产目录页——按契约渲染目录树（结构即界面）
///
/// 导航模型：
/// 1. 点击节点导航到子节点视图（而非展开/收起）
/// 2. 显示面包屑导航，点击可返回对应层级
/// 3. 每个节点只显示其直接子节点和文件
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
  final List<String> _navigationStack = []; // 导航栈，记录访问过的节点名称

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
      return Scaffold(
        appBar: AppBar(
          title: Text('${widget.label}资产'),
          leading: IconButton(
            icon: const Icon(Icons.arrow_back),
            onPressed: () => Navigator.of(context).pop(),
          ),
        ),
        body: Padding(
          padding: const EdgeInsets.all(28),
          child: Text(_error!, style: const TextStyle(color: Color(0xFFEF4444))),
        ),
      );
    }
    if (_catalog == null) {
      return const Scaffold(
        body: Center(child: CircularProgressIndicator()),
      );
    }
    final catalog = _catalog!;
    final currentNodes = _getCurrentNodes();
    final currentFiles = _getCurrentFiles();
    
    // 构建标题文本
    String titleText = '${catalog.contract.label}资产';
    if (_navigationStack.isNotEmpty) {
      titleText = _navigationStack.last;
    }
    
    // 构建描述文本
    String descriptionText;
    if (_navigationStack.isEmpty) {
      descriptionText = catalog.nodes.isEmpty
          ? '数据源不可用（Web 无文件系统）'
          : '${catalog.contract.asset} · ${catalog.nodes.length} 个一级目录';
    } else {
      final totalItems = currentNodes.length + currentFiles.length;
      descriptionText = '当前层级 · $totalItems 个子项';
    }
    
    return Scaffold(
      appBar: AppBar(
        title: Text(titleText),
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => Navigator.of(context).pop(),
        ),
      ),
      body: Padding(
        padding: const EdgeInsets.fromLTRB(28, 0, 28, 0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // 面包屑导航（内部导航）
            if (_navigationStack.isNotEmpty)
              _buildBreadcrumb(),
            Text(
              descriptionText,
              style: const TextStyle(fontSize: 13, color: Color(0xFF94A3B8)),
            ),
            const SizedBox(height: 20),
            Expanded(
              child: ListView(
                children: [
                  // 显示子节点
                  ...currentNodes.map((node) => _NodeTile(
                        node: node,
                        onNavigateToNode: (nodeName) => _navigateToNode(nodeName),
                        onOpenFile: (file) => _openFile(file),
                      )),
                  // 显示文件
                  ...currentFiles.map((file) => _FileTile(
                        file: file,
                        onTap: () => _openFile(file),
                      )),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  void _openFile(CatalogFile file) {
    // 阅读详情（只读）——显示文件名提示；正文阅读后续实现
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text('${file.title}（${file.path}）')),
    );
  }

  void _navigateToNode(String nodeName) {
    // 导航到指定节点
    setState(() {
      _navigationStack.add(nodeName);
    });
  }

  void _navigateBack() {
    // 返回上一级
    if (_navigationStack.isNotEmpty) {
      setState(() {
        _navigationStack.removeLast();
      });
    }
  }

  void _navigateToRoot() {
    // 返回根目录
    setState(() {
      _navigationStack.clear();
    });
  }

  // 获取当前应该显示的节点列表
  List<CatalogNode> _getCurrentNodes() {
    if (_catalog == null) return [];
    
    List<CatalogNode> currentNodes = _catalog!.nodes;
    
    // 根据导航栈逐级深入
    for (String nodeName in _navigationStack) {
      CatalogNode? foundNode;
      for (CatalogNode node in currentNodes) {
        if (node.name == nodeName) {
          foundNode = node;
          break;
        }
      }
      
      if (foundNode != null && foundNode.children.isNotEmpty) {
        currentNodes = foundNode.children;
      } else {
        // 如果找不到节点或没有子节点，停止深入
        break;
      }
    }
    
    return currentNodes;
  }

  // 获取当前应该显示的文件列表
  List<CatalogFile> _getCurrentFiles() {
    if (_catalog == null) return [];
    
    List<CatalogNode> currentNodes = _catalog!.nodes;
    CatalogNode? currentNode;
    
    // 根据导航栈逐级深入，找到当前节点
    for (String nodeName in _navigationStack) {
      CatalogNode? foundNode;
      for (CatalogNode node in currentNodes) {
        if (node.name == nodeName) {
          foundNode = node;
          break;
        }
      }
      
      if (foundNode != null) {
        currentNode = foundNode;
        if (foundNode.children.isNotEmpty) {
          currentNodes = foundNode.children;
        } else {
          break;
        }
      } else {
        break;
      }
    }
    
    // 返回当前节点的文件
    return currentNode?.files ?? [];
  }

  Widget _buildBreadcrumb() {
    return Container(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        children: [
          // 返回按钮
          InkWell(
            onTap: _navigateBack,
            child: const Row(
              children: [
                Icon(Icons.arrow_back, size: 16, color: Color(0xFF4F46E5)),
                SizedBox(width: 4),
                Text(
                  '返回',
                  style: TextStyle(
                    fontSize: 12,
                    color: Color(0xFF4F46E5),
                    fontWeight: FontWeight.w500,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(width: 16),
          // 面包屑路径
          Expanded(
            child: SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: Row(
                children: [
                  // 根目录
                  InkWell(
                    onTap: _navigateToRoot,
                    child: Text(
                      widget.label,
                      style: const TextStyle(
                        fontSize: 12,
                        color: Color(0xFF4F46E5),
                        fontWeight: FontWeight.w500,
                      ),
                    ),
                  ),
                  // 遍历导航栈
                  for (int i = 0; i < _navigationStack.length; i++) ...[
                    const Padding(
                      padding: EdgeInsets.symmetric(horizontal: 4),
                      child: Icon(Icons.chevron_right, size: 12, color: Color(0xFF94A3B8)),
                    ),
                    InkWell(
                      onTap: () {
                        // 导航到指定层级
                        setState(() {
                          _navigationStack.removeRange(i, _navigationStack.length);
                        });
                      },
                      child: Text(
                        _navigationStack[i],
                        style: const TextStyle(
                          fontSize: 12,
                          color: Color(0xFF4F46E5),
                          fontWeight: FontWeight.w500,
                        ),
                      ),
                    ),
                  ],
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// 目录节点卡片（点击导航到子节点）
class _NodeTile extends StatelessWidget {
  final CatalogNode node;
  final void Function(String nodeName) onNavigateToNode;
  final void Function(CatalogFile file) onOpenFile;

  const _NodeTile({
    required this.node,
    required this.onNavigateToNode,
    required this.onOpenFile,
  });

  @override
  Widget build(BuildContext context) {
    final hasChildren = node.children.isNotEmpty;
    final fileCount = node.files.length +
        node.children.fold<int>(0, (sum, c) => sum + c.files.length);
    
    return Container(
      margin: const EdgeInsets.only(bottom: 8),
      child: Material(
        color: Colors.white,
        borderRadius: BorderRadius.circular(12),
        child: InkWell(
          onTap: hasChildren ? () => onNavigateToNode(node.name) : null,
          borderRadius: BorderRadius.circular(12),
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
            child: Row(
              children: [
                Icon(
                  hasChildren ? Icons.folder_outlined : Icons.description_outlined,
                  size: 18,
                  color: const Color(0xFF4F46E5),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        node.name,
                        style: const TextStyle(
                          fontSize: 14,
                          fontWeight: FontWeight.w600,
                          color: Color(0xFF1E293B),
                        ),
                      ),
                      if (node.label.isNotEmpty)
                        Text(
                          node.label,
                          style: const TextStyle(
                            fontSize: 11,
                            color: Color(0xFF94A3B8),
                          ),
                        ),
                    ],
                  ),
                ),
                Text(
                  '$fileCount 项',
                  style: const TextStyle(fontSize: 11, color: Color(0xFF94A3B8)),
                ),
                if (hasChildren)
                  const Icon(
                    Icons.chevron_right,
                    size: 18,
                    color: Color(0xFF94A3B8),
                  ),
              ],
            ),
          ),
        ),
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
    return Container(
      margin: const EdgeInsets.only(bottom: 4),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
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
        ),
      ),
    );
  }
}