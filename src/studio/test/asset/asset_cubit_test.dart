import 'dart:io';
import 'dart:math';

import 'package:flutter_test/flutter_test.dart';
import 'package:qtfounder_studio/asset/bloc/asset_cubit.dart';
import 'package:qtfounder_studio/asset/data/asset_repository.dart';
import 'package:qtfounder_studio/asset/model/asset_file.dart';

class _FakeRepository extends AssetRepository {
  _FakeRepository(this.assets, {this.failOn = const {}})
    : super(root: Directory.systemTemp, trashDir: Directory.systemTemp);

  final List<AssetFile> assets;
  final Set<String> failOn;
  final List<String> moved = [];

  @override
  Future<List<AssetFile>> scan() async => assets;

  @override
  Future<String> preview(
    AssetFile asset, {
    int maxLines = 40,
    int maxBytes = 4000,
  }) async => '预览 ${asset.id}';

  @override
  Future<String> moveToTrash(AssetFile asset) async {
    if (failOn.contains(asset.id)) {
      throw FileSystemException('磁盘只读', asset.path);
    }
    moved.add(asset.id);
    return asset.path;
  }
}

class _FailingRepository extends AssetRepository {
  _FailingRepository()
    : super(
        root: Directory('/nonexistent/repo'),
        trashDir: Directory('/nonexistent/repo/.trash'),
      );

  @override
  Future<List<AssetFile>> scan() async {
    throw AssetScanException('仓库根目录不存在：${root.path}');
  }
}

AssetFile asset(int index, {AssetKind kind = AssetKind.document}) => AssetFile(
  id: 'docs/a$index.md',
  path: '/repo/docs/a$index.md',
  kind: kind,
  size: 10,
  modified: DateTime(2026, 1, 1),
);

void main() {
  test('扫描为空进入 AssetEmpty', () async {
    final cubit = AssetCubit(_FakeRepository(const []));
    await cubit.load();

    expect(cubit.state, isA<AssetEmpty>());
    await cubit.close();
  });

  test('扫描失败进入 AssetError 并带上路径', () async {
    final cubit = AssetCubit(_FailingRepository());
    await cubit.load();

    expect(
      cubit.state,
      isA<AssetError>().having(
        (state) => state.message,
        'message',
        contains('/nonexistent/repo'),
      ),
    );
    await cubit.close();
  });

  test('批次不超过 batchSize', () async {
    final cubit = AssetCubit(
      _FakeRepository([for (var i = 0; i < 30; i++) asset(i)]),
      batchSize: 20,
      random: Random(7),
    );
    await cubit.load();

    expect(cubit.state, isA<AssetReady>());
    expect((cubit.state as AssetReady).assets, hasLength(20));
    await cubit.close();
  });

  test('批次带预览', () async {
    final cubit = AssetCubit(_FakeRepository([asset(1)]), random: Random(1));
    await cubit.load();

    final ready = cubit.state as AssetReady;
    expect(ready.previews[asset(1).id], '预览 ${asset(1).id}');
    await cubit.close();
  });

  test('决策走完进 AssetConfirm', () async {
    final cubit = AssetCubit(
      _FakeRepository([asset(1), asset(2)]),
      random: Random(1),
    );
    await cubit.load();

    cubit.decide(Decision.delete);
    expect(cubit.state, isA<AssetReady>());
    expect((cubit.state as AssetReady).current, 1);

    cubit.decide(Decision.keep);
    final confirm = cubit.state;
    expect(confirm, isA<AssetConfirm>());
    expect((confirm as AssetConfirm).marked, hasLength(1));
    await cubit.close();
  });

  test('确认只移动标记删除的资产，失败的进清单', () async {
    final repository = _FakeRepository(
      [asset(1), asset(2), asset(3)],
      failOn: {asset(2).id},
    );
    final cubit = AssetCubit(repository, random: Random(3));
    await cubit.load();

    for (final target in (cubit.state as AssetReady).assets) {
      cubit.decide(target.id == asset(2).id ? Decision.delete : Decision.keep);
    }
    expect(cubit.state, isA<AssetConfirm>());

    await cubit.confirm();

    final done = cubit.state;
    expect(done, isA<AssetDone>());
    done as AssetDone;
    expect(done.succeeded, isEmpty);
    expect(done.failed, hasLength(1));
    expect(done.failed.single.asset.id, asset(2).id);
    expect(done.failed.single.message, contains('磁盘只读'));
    expect(repository.moved, isEmpty);
    await cubit.close();
  });

  test('确认后成功的进 succeeded', () async {
    final repository = _FakeRepository([asset(1), asset(2)]);
    final cubit = AssetCubit(repository, random: Random(5));
    await cubit.load();

    cubit.decide(Decision.delete);
    cubit.decide(Decision.keep);
    await cubit.confirm();

    final done = cubit.state as AssetDone;
    expect(done.succeeded, hasLength(1));
    expect(done.failed, isEmpty);
    expect(repository.moved, hasLength(1));
    await cubit.close();
  });
}
