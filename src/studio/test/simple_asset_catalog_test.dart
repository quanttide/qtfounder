/// 简化的资产目录页面测试
library;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:qtfounder_studio/models/asset_catalog.dart';

// 创建一个简单的测试组件，模拟AssetCatalogScreen的导航逻辑
class TestAssetCatalogScreen extends StatefulWidget {
  final AssetCatalog catalog;

  const TestAssetCatalogScreen({super.key, required this.catalog});

  @override
  State<TestAssetCatalogScreen> createState() => _TestAssetCatalogScreenState();
}

class _TestAssetCatalogScreenState extends State<TestAssetCatalogScreen> {
  final List<String> _navigationStack = [];

  @override
  Widget build(BuildContext context) {
    final currentNodes = _getCurrentNodes();
    final currentFiles = _getCurrentFiles();
    
    return Scaffold(
      appBar: AppBar(
        title: Text(_navigationStack.isEmpty ? '资产' : _navigationStack.last),
        leading: _navigationStack.isNotEmpty
            ? IconButton(
                icon: const Icon(Icons.arrow_back),
                onPressed: () {
                  setState(() {
                    _navigationStack.removeLast();
                  });
                },
              )
            : null,
      ),
      body: Column(
        children: [
          // 面包屑导航
          if (_navigationStack.isNotEmpty)
            _buildBreadcrumb(),
          Expanded(
            child: ListView(
              children: [
                // 显示子节点
                ...currentNodes.map((node) => ListTile(
                      leading: const Icon(Icons.folder),
                      title: Text(node.name),
                      subtitle: Text('${node.children.length + node.files.length} 项'),
                      onTap: () {
                        setState(() {
                          _navigationStack.add(node.name);
                        });
                      },
                    )),
                // 显示文件
                ...currentFiles.map((file) => ListTile(
                      leading: const Icon(Icons.description),
                      title: Text(file.title),
                      onTap: () {
                        // 模拟打开文件
                        ScaffoldMessenger.of(context).showSnackBar(
                          SnackBar(content: Text('${file.title}（${file.path}）')),
                        );
                      },
                    )),
              ],
            ),
          ),
        ],
      ),
    );
  }

  List<CatalogNode> _getCurrentNodes() {
    List<CatalogNode> currentNodes = widget.catalog.nodes;
    
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
        break;
      }
    }
    
    return currentNodes;
  }

  List<CatalogFile> _getCurrentFiles() {
    List<CatalogNode> currentNodes = widget.catalog.nodes;
    CatalogNode? currentNode;
    
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
    
    return currentNode?.files ?? [];
  }

  Widget _buildBreadcrumb() {
    return Container(
      padding: const EdgeInsets.all(8),
      child: Row(
        children: [
          InkWell(
            onTap: () {
              setState(() {
                _navigationStack.clear();
              });
            },
            child: const Text('资产'),
          ),
          for (int i = 0; i < _navigationStack.length; i++) ...[
            const Icon(Icons.chevron_right, size: 16),
            InkWell(
              onTap: () {
                setState(() {
                  _navigationStack.removeRange(i, _navigationStack.length);
                });
              },
              child: Text(_navigationStack[i]),
            ),
          ],
        ],
      ),
    );
  }
}

void main() {
  group('简化资产目录页面测试', () {
    testWidgets('初始状态显示根节点', (WidgetTester tester) async {
      final catalog = AssetCatalog(
        contract: const AssetContract(
          asset: 'fiction',
          label: '小说',
          root: 'fiction',
          ignore: [],
          levels: [],
          naming: NamingSpec(unsorted: 'natural'),
        ),
        nodes: [
          const CatalogNode(
            name: '职场言情',
            label: '小说',
            children: [],
            files: [],
          ),
          const CatalogNode(
            name: '校园言情',
            label: '小说',
            children: [],
            files: [],
          ),
        ],
      );

      await tester.pumpWidget(
        MaterialApp(
          home: TestAssetCatalogScreen(catalog: catalog),
        ),
      );

      // 验证初始状态
      expect(find.text('资产'), findsOneWidget);
      expect(find.text('职场言情'), findsWidgets);
      expect(find.text('校园言情'), findsWidgets);
      expect(find.byIcon(Icons.arrow_back), findsNothing); // 没有返回按钮
    });

    testWidgets('点击节点导航到子节点', (WidgetTester tester) async {
      final catalog = AssetCatalog(
        contract: const AssetContract(
          asset: 'fiction',
          label: '小说',
          root: 'fiction',
          ignore: [],
          levels: [],
          naming: NamingSpec(unsorted: 'natural'),
        ),
        nodes: [
          const CatalogNode(
            name: '职场言情',
            label: '小说',
            children: [
              CatalogNode(
                name: '4_改稿',
                label: '阶段',
                children: [],
                files: [],
              ),
            ],
            files: [],
          ),
        ],
      );

      await tester.pumpWidget(
        MaterialApp(
          home: TestAssetCatalogScreen(catalog: catalog),
        ),
      );

      // 点击"职场言情"节点
      await tester.tap(find.text('职场言情'));
      await tester.pumpAndSettle();

      // 验证导航到子节点
      expect(find.text('职场言情'), findsWidgets); // 标题变为"职场言情"
      expect(find.text('4_改稿'), findsWidgets); // 显示子节点
      expect(find.byIcon(Icons.arrow_back), findsOneWidget); // 显示返回按钮
    });

    testWidgets('点击返回按钮返回上一级', (WidgetTester tester) async {
      final catalog = AssetCatalog(
        contract: const AssetContract(
          asset: 'fiction',
          label: '小说',
          root: 'fiction',
          ignore: [],
          levels: [],
          naming: NamingSpec(unsorted: 'natural'),
        ),
        nodes: [
          const CatalogNode(
            name: '职场言情',
            label: '小说',
            children: [
              CatalogNode(
                name: '4_改稿',
                label: '阶段',
                children: [],
                files: [],
              ),
            ],
            files: [],
          ),
        ],
      );

      await tester.pumpWidget(
        MaterialApp(
          home: TestAssetCatalogScreen(catalog: catalog),
        ),
      );

      // 先导航到子节点
      await tester.tap(find.text('职场言情'));
      await tester.pumpAndSettle();

      // 验证在子节点页面
      expect(find.text('职场言情'), findsWidgets);
      expect(find.text('4_改稿'), findsWidgets);

      // 点击返回按钮
      await tester.tap(find.byIcon(Icons.arrow_back));
      await tester.pumpAndSettle();

      // 验证返回到根节点
      expect(find.text('资产'), findsOneWidget);
      expect(find.text('职场言情'), findsWidgets);
      expect(find.byIcon(Icons.arrow_back), findsNothing); // 没有返回按钮
    });

    testWidgets('面包屑导航可以返回指定层级', (WidgetTester tester) async {
      final catalog = AssetCatalog(
        contract: const AssetContract(
          asset: 'fiction',
          label: '小说',
          root: 'fiction',
          ignore: [],
          levels: [],
          naming: NamingSpec(unsorted: 'natural'),
        ),
        nodes: [
          const CatalogNode(
            name: '职场言情',
            label: '小说',
            children: [
              CatalogNode(
                name: '4_改稿',
                label: '阶段',
                children: [
                  CatalogNode(
                    name: '子章节',
                    label: '章节',
                    children: [],
                    files: [],
                  ),
                ],
                files: [],
              ),
            ],
            files: [],
          ),
        ],
      );

      await tester.pumpWidget(
        MaterialApp(
          home: TestAssetCatalogScreen(catalog: catalog),
        ),
      );

      // 导航到第三层
      await tester.tap(find.text('职场言情'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('4_改稿'));
      await tester.pumpAndSettle();

      // 验证在第三层
      expect(find.text('4_改稿'), findsWidgets);
      expect(find.text('子章节'), findsWidgets);

      // 点击面包屑中的"资产"返回根目录
      await tester.tap(find.text('资产'));
      await tester.pumpAndSettle();

      // 验证返回到根节点
      expect(find.text('资产'), findsOneWidget);
      expect(find.text('职场言情'), findsWidgets);
    });

    testWidgets('点击文件显示SnackBar', (WidgetTester tester) async {
      final catalog = AssetCatalog(
        contract: const AssetContract(
          asset: 'fiction',
          label: '小说',
          root: 'fiction',
          ignore: [],
          levels: [],
          naming: NamingSpec(unsorted: 'natural'),
        ),
        nodes: [
          const CatalogNode(
            name: '职场言情',
            label: '小说',
            children: [],
            files: [
              CatalogFile(
                name: '1_1_咖啡厅重逢.md',
                path: '/test/1_1_咖啡厅重逢.md',
                title: '咖啡厅重逢',
                sortKey: '1_1',
                version: 1,
              ),
            ],
          ),
        ],
      );

      await tester.pumpWidget(
        MaterialApp(
          home: TestAssetCatalogScreen(catalog: catalog),
        ),
      );

      // 先点击节点进入子页面
      await tester.tap(find.text('职场言情'));
      await tester.pumpAndSettle();

      // 现在应该能看到文件
      expect(find.text('咖啡厅重逢'), findsWidgets);
      
      // 点击文件
      await tester.tap(find.text('咖啡厅重逢'));
      await tester.pumpAndSettle();

      // 验证显示SnackBar
      expect(find.text('咖啡厅重逢（/test/1_1_咖啡厅重逢.md）'), findsOneWidget);
    });
  });
}