import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:qtfounder_studio/data/models/journal_segment.dart';
import 'package:qtfounder_studio/data/repos/journal_repository.dart';
import 'package:qtfounder_studio/data/states/data_cubit.dart';

class _FakeRepository extends JournalRepository {
  _FakeRepository(this.items, {this.failOnSave = false})
    : super(root: Directory.systemTemp);

  final List<JournalSegment> items;
  final bool failOnSave;
  final List<List<Annotation>> saved = [];

  @override
  Future<List<JournalSegment>> segments() async => items;

  @override
  Future<String> save(List<Annotation> annotations) async {
    if (failOnSave) throw JournalScanException('磁盘只读');
    saved.add(annotations);
    return '/tmp/journal.jsonl';
  }
}

List<JournalSegment> _segments(int count) => [
  for (var i = 0; i < count; i++)
    JournalSegment(
      id: 's$i',
      path: '/tmp/s$i.md',
      domain: 'default',
      text: '第 $i 段',
    ),
];

void main() {
  test('批次 3 段，标 2 跳 1 进保存页', () async {
    final cubit = DataCubit(_FakeRepository(_segments(3)));
    await cubit.load();

    final ready = cubit.state as DataReady;
    final first = ready.segment.id;
    cubit.label(AnnotateLabel.fact);
    final skipped = (cubit.state as DataReady).segment.id;
    cubit.skip();
    cubit.label(AnnotateLabel.action);

    final save = cubit.state as DataSave;
    expect(save.annotations, hasLength(2));
    expect(save.annotations[first]!.label, AnnotateLabel.fact);
    expect(save.annotations, isNot(contains(skipped)), reason: '跳过的没标');
    await cubit.close();
  });

  test('save 落盘后进完成页，条数正确', () async {
    final repository = _FakeRepository(_segments(2));
    final cubit = DataCubit(repository);
    await cubit.load();

    cubit.label(AnnotateLabel.need);
    cubit.label(AnnotateLabel.need);
    await cubit.save();

    final done = cubit.state as DataDone;
    expect(done.count, 2);
    expect(done.saved, '/tmp/journal.jsonl');
    expect(repository.saved.single, hasLength(2));
    await cubit.close();
  });

  test('保存失败报错', () async {
    final cubit = DataCubit(_FakeRepository(_segments(1), failOnSave: true));
    await cubit.load();

    cubit.label(AnnotateLabel.feeling);
    await cubit.save();

    final error = cubit.state as DataError;
    expect(error.message, '磁盘只读');
    await cubit.close();
  });

  test('空仓库进空态', () async {
    final cubit = DataCubit(_FakeRepository(const []));
    await cubit.load();

    expect(cubit.state, isA<DataEmpty>());
    await cubit.close();
  });

  test('一条都没标就 save，回加载重新抽', () async {
    final repository = _FakeRepository(_segments(1));
    final cubit = DataCubit(repository);
    await cubit.load();

    cubit.skip();
    expect(cubit.state, isA<DataSave>());
    await cubit.save();

    expect(cubit.state, isA<DataReady>());
    expect(repository.saved, isEmpty);
    await cubit.close();
  });
}
