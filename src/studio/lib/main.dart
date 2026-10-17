import 'package:flutter/material.dart';

void main() {
  runApp(const FounderApp());
}

/// 应用入口与骨架。
///
/// 业务代码已归档至 `data/archive/platform/qtfounder/studio/`；重建时在 `lib/`
/// 下按 bloc / models / repositories / screens / services 分层添加。
class FounderApp extends StatelessWidget {
  const FounderApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: '量潮创始人工作台',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF4F46E5),
          surface: Colors.white,
          brightness: Brightness.light,
        ),
        scaffoldBackgroundColor: const Color(0xFFF1F5F9),
        useMaterial3: true,
        fontFamily: 'Inter',
      ),
      home: const Scaffold(body: Center(child: Text('量潮创始人工作台'))),
    );
  }
}
