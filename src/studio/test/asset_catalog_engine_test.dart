import 'package:flutter_test/flutter_test.dart';

import 'package:qtfounder_studio/data/asset_catalog_engine.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  // 真实数据源（测试环境路径：src/studio → quanttide-founder 根 = 4 级）
  const repoRoot = '../../../../';

  test('fiction 契约：编号=排序，1_1 在 10_1 前，未排序在后', () async {
    final engine = AssetCatalogEngine(
      contractAssetPath: 'assets/contracts/fiction.yaml',
      dataSourceRoot: '$repoRoot/assets/fiction',
    );
    final catalog = await engine.load();

    expect(catalog.contract.label, '小说');
    // 一级 = 三本小说
    final novels = catalog.nodes.map((n) => n.name).toList();
    expect(novels, containsAll(['职场言情', '校园言情', '重生言情']));

    // 职场言情 → 阶段 → 文件
    final zc = catalog.nodes.firstWhere((n) => n.name == '职场言情');
    final stageNames = zc.children.map((s) => s.name).toList();
    expect(stageNames, containsAll(['1_灵感', '2_脚本', '3_初稿', '4_改稿']));

    final gaogao = zc.children.firstWhere((s) => s.name == '4_改稿');
    final files = gaogao.files;
    expect(files.length, 19);
    // 编号排序：1_1 第一，10_1 在 1_1 之后（数值而非字典序）
    expect(files.first.title, '咖啡厅重逢');
    expect(files.first.sortKey, '1_1');
    final idx10 = files.indexWhere((f) => f.sortKey == '10_1');
    expect(idx10, greaterThan(files.indexWhere((f) => f.sortKey == '1_1')));

    // 未排序（无编号）在末尾：2_脚本 中"偷看睡觉"无编号
    final jiaoben = zc.children.firstWhere((s) => s.name == '2_脚本');
    expect(jiaoben.files.last.title, '雨伞同行'); // 自然序最后（无编号组）
    expect(jiaoben.files.where((f) => f.sortKey == null).isNotEmpty, true);
  });

  test('memory 契约：自由命名 + journal 二级目录', () async {
    final engine = AssetCatalogEngine(
      contractAssetPath: 'assets/contracts/memory.yaml',
      dataSourceRoot: '$repoRoot/assets/memory',
    );
    final catalog = await engine.load();

    expect(catalog.contract.label, '记忆');
    final categories = catalog.nodes.map((n) => n.name).toList();
    expect(categories, containsAll(['context', 'intention', 'journal', 'profile', 'report', 'roadmap']));

    // journal 有二级（default/），其他类型直接文件
    final journal = catalog.nodes.firstWhere((n) => n.name == 'journal');
    expect(journal.children.map((c) => c.name), contains('default'));
    final ctx = catalog.nodes.firstWhere((n) => n.name == 'context');
    expect(ctx.files.map((f) => f.title), contains('fiction-adaptation'));
    // 仓库级文件被 ignore（不进树）
    expect(ctx.files.map((f) => f.name), isNot(contains('README.md')));
  });
}
