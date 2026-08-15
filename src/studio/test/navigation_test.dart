/// 导航逻辑单元测试
library;

import 'package:flutter_test/flutter_test.dart';

void main() {
  group('导航栈逻辑', () {
    test('导航栈基本操作', () {
      final List<String> navigationStack = [];

      // 测试添加节点
      navigationStack.add('节点1');
      expect(navigationStack, ['节点1']);

      navigationStack.add('节点2');
      expect(navigationStack, ['节点1', '节点2']);

      // 测试返回上一级
      navigationStack.removeLast();
      expect(navigationStack, ['节点1']);

      // 测试返回根目录
      navigationStack.clear();
      expect(navigationStack, isEmpty);
    });

    test('导航到指定层级', () {
      final List<String> navigationStack = ['节点1', '节点2', '节点3'];

      // 导航到节点2（移除节点3）
      navigationStack.removeRange(2, navigationStack.length);
      expect(navigationStack, ['节点1', '节点2']);

      // 导航到根目录
      navigationStack.removeRange(0, navigationStack.length);
      expect(navigationStack, isEmpty);
    });

    test('获取当前节点列表', () {
      // 模拟节点结构
      final Map<String, List<String>> nodeChildren = {
        '根节点': ['节点1', '节点2'],
        '节点1': ['子节点1_1', '子节点1_2'],
        '节点2': ['子节点2_1'],
        '子节点1_1': [],
      };

      // 测试获取根节点的子节点
      List<String> getCurrentNodes(List<String> navigationStack) {
        if (navigationStack.isEmpty) {
          return nodeChildren['根节点'] ?? [];
        }

        String currentNodeName = navigationStack.last;
        return nodeChildren[currentNodeName] ?? [];
      }

      // 测试根节点
      expect(getCurrentNodes([]), ['节点1', '节点2']);

      // 测试节点1
      expect(getCurrentNodes(['节点1']), ['子节点1_1', '子节点1_2']);

      // 测试节点2
      expect(getCurrentNodes(['节点2']), ['子节点2_1']);

      // 测试子节点1_1（没有子节点）
      expect(getCurrentNodes(['节点1', '子节点1_1']), []);
    });

    test('获取当前文件列表', () {
      // 模拟文件结构
      final Map<String, List<String>> nodeFiles = {
        '根节点': [],
        '节点1': ['文件1_1.md', '文件1_2.md'],
        '节点2': ['文件2_1.md'],
        '子节点1_1': ['文件1_1_1.md'],
      };

      // 测试获取当前文件列表
      List<String> getCurrentFiles(List<String> navigationStack) {
        if (navigationStack.isEmpty) {
          return nodeFiles['根节点'] ?? [];
        }

        String currentNodeName = navigationStack.last;
        return nodeFiles[currentNodeName] ?? [];
      }

      // 测试根节点
      expect(getCurrentFiles([]), isEmpty);

      // 测试节点1
      expect(getCurrentFiles(['节点1']), ['文件1_1.md', '文件1_2.md']);

      // 测试节点2
      expect(getCurrentFiles(['节点2']), ['文件2_1.md']);

      // 测试子节点1_1
      expect(getCurrentFiles(['节点1', '子节点1_1']), ['文件1_1_1.md']);
    });
  });
}