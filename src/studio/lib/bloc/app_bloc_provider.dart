import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../repositories/analysis_repository.dart';
import '../repositories/chapter_repository.dart';
import '../repositories/file_chapter_repository.dart';
import '../services/llm_client.dart';
import 'analyze/analyze_bloc.dart';
import 'chapter_list/chapter_list_bloc.dart';
import 'editor/editor_bloc.dart';
import 'workflow/workflow_bloc.dart';

/// 应用级 Bloc Provider
class AppBlocProvider extends StatelessWidget {
  final Widget child;
  final ChapterRepository? chapterRepository;

  const AppBlocProvider({
    super.key,
    required this.child,
    this.chapterRepository,
  });

  @override
  Widget build(BuildContext context) {
    return MultiRepositoryProvider(
      providers: [
        RepositoryProvider<ChapterRepository>(
          create: (context) => chapterRepository ?? _createDefaultRepository(),
        ),
        RepositoryProvider<ChapterAnalysisRepository>(
          create: (context) => FileAnalysisRepository(),
        ),
        RepositoryProvider<LLMClient>(
          create: (context) => LLMClient(config: LLMConfig.defaults()),
        ),
      ],
      child: MultiBlocProvider(
        providers: [
          BlocProvider<ChapterListBloc>(
            create: (context) => ChapterListBloc(
              repository: context.read<ChapterRepository>(),
            )..add(const LoadChapters()),
          ),
          BlocProvider<EditorBloc>(
            create: (context) => EditorBloc(
              repository: context.read<ChapterRepository>(),
            ),
          ),
          BlocProvider<WorkflowBloc>(
            create: (context) => WorkflowBloc(
              repository: context.read<ChapterRepository>(),
            )..add(const LoadWorkflow()),
          ),
          BlocProvider<AnalyzeBloc>(
            create: (context) => AnalyzeBloc(
              chapterRepository: context.read<ChapterRepository>(),
              analysisRepository: context.read<ChapterAnalysisRepository>(),
              llm: context.read<LLMClient>(),
            ),
          ),
        ],
        child: child,
      ),
    );
  }

  ChapterRepository _createDefaultRepository() {
    final home = Platform.environment['HOME'] ?? '';
    final basePath = '$home/repos/quanttide-founder/assets/fiction';
    return FileChapterRepository(basePath: basePath);
  }
}

/// 页面级 Bloc Provider（用于测试或独立页面）
class PageBlocProvider extends StatelessWidget {
  final Widget child;
  final ChapterRepository repository;

  const PageBlocProvider({
    super.key,
    required this.child,
    required this.repository,
  });

  @override
  Widget build(BuildContext context) {
    return MultiBlocProvider(
      providers: [
        BlocProvider<ChapterListBloc>(
          create: (context) => ChapterListBloc(
            repository: repository,
          )..add(const LoadChapters()),
        ),
        BlocProvider<EditorBloc>(
          create: (context) => EditorBloc(
            repository: repository,
          ),
        ),
        BlocProvider<WorkflowBloc>(
          create: (context) => WorkflowBloc(
            repository: repository,
          )..add(const LoadWorkflow()),
        ),
      ],
      child: child,
    );
  }
}
