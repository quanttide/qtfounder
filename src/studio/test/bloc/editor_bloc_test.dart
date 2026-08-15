import 'package:flutter_test/flutter_test.dart';
import 'package:qtfounder_studio/bloc/editor/editor_bloc.dart';

import 'mock_chapter_repository.dart';

void main() {
  group('EditorBloc', () {
    late MockChapterRepository repository;
    late EditorBloc bloc;

    setUp(() {
      repository = MockChapterRepository();
      bloc = EditorBloc(repository: repository);
    });

    tearDown(() {
      bloc.close();
    });

    test('initial state is correct', () {
      expect(bloc.state, equals(EditorState.initial()));
    });

    test('UpdateContent updates content and sets isDirty', () async {
      bloc.add(const UpdateContent('Hello World'));
      await bloc.stream.first;
      
      expect(bloc.state.content, equals('Hello World'));
      expect(bloc.state.isDirty, isTrue);
      expect(bloc.state.saveStatus, equals(SaveStatus.unsaved));
    });

    test('LoadContent loads chapter content', () async {
      bloc.add(const LoadContent('test_chapter_1'));
      await bloc.stream.where((state) => !state.isLoading).first;
      
      expect(bloc.state.chapterId, equals('test_chapter_1'));
      expect(bloc.state.content, contains('测试章节1'));
      expect(bloc.state.isDirty, isFalse);
      expect(bloc.state.saveStatus, equals(SaveStatus.saved));
    });

    test('calculateWordCount counts words correctly', () {
      expect(EditorState.calculateWordCount(''), equals(0));
      expect(EditorState.calculateWordCount('Hello'), equals(1));
      expect(EditorState.calculateWordCount('Hello World'), equals(2));
      expect(EditorState.calculateWordCount('# Title'), equals(1));
      expect(EditorState.calculateWordCount('bold text'), equals(2));
    });

    test('ClearEditor resets state', () async {
      // 先加载内容
      bloc.add(const LoadContent('test_chapter_1'));
      await bloc.stream.where((state) => !state.isLoading).first;
      
      // 清除
      bloc.add(const ClearEditor());
      await bloc.stream.first;
      
      expect(bloc.state.chapterId, isNull);
      expect(bloc.state.content, isEmpty);
      expect(bloc.state.isDirty, isFalse);
    });
  });
}