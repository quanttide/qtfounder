/// LLM 客户端——OpenAI 兼容协议（DeepSeek API）。
///
/// API key 从运行时环境变量 QTFOUNDER_LLM_API_KEY 读取（桌面端）：
/// ```bash
/// QTFOUNDER_LLM_API_KEY=sk-xxx flutter run -d linux
/// ```
/// 可通过 QTFOUNDER_LLM_BASE_URL / QTFOUNDER_LLM_MODEL（环境变量或 dart-define）覆盖。
library;

import 'dart:convert';
import 'dart:io' show Platform;

import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:http/http.dart' as http;

import '../models/analysis.dart';

/// LLM 配置
class LLMConfig {
  final String baseUrl;
  final String model;
  final String apiKey;

  const LLMConfig({
    required this.baseUrl,
    required this.model,
    this.apiKey = '',
  });

  /// 默认配置：DeepSeek API（https://api.deepseek.com/v1，模型 deepseek-v4-flash）。
  /// API key 必须通过 QTFOUNDER_LLM_API_KEY 提供。
  factory LLMConfig.defaults() {
    const dartUrl = String.fromEnvironment('QTFOUNDER_LLM_BASE_URL');
    const dartModel = String.fromEnvironment('QTFOUNDER_LLM_MODEL');

    // 运行时环境变量（桌面端；Web 无环境变量，回退 dart-define）
    final apiKey = kIsWeb ? '' : (Platform.environment['QTFOUNDER_LLM_API_KEY'] ?? '');
    final envUrl = kIsWeb ? null : Platform.environment['QTFOUNDER_LLM_BASE_URL'];
    final envModel = kIsWeb ? null : Platform.environment['QTFOUNDER_LLM_MODEL'];

    return LLMConfig(
      baseUrl: (envUrl ?? dartUrl).isNotEmpty ? (envUrl ?? dartUrl) : 'https://api.deepseek.com/v1',
      model: (envModel ?? dartModel).isNotEmpty ? (envModel ?? dartModel) : 'deepseek-v4-flash',
      apiKey: apiKey,
    );
  }
}

/// LLM 客户端
class LLMClient {
  final LLMConfig config;
  final http.Client _client;

  LLMClient({required this.config, http.Client? client})
      : _client = client ?? http.Client();

  /// 分析文本结构，返回结构化整理结果。
  /// 协议约束：输出纯 JSON，绝不返回改写后的文本。
  /// [stageId] 决定分析方法：0_日志 → 灵感分解；其他阶段 → 结构分析。
  Future<ChapterAnalysis> analyzeStructure({
    required String chapterId,
    required String chapterPath,
    required String stageId,
    required String content,
    required List<String> previousSuggestions,
  }) async {
    final prompt = _buildPrompt(stageId, content, previousSuggestions);
    final body = jsonEncode({
      'model': config.model,
      'messages': [
        {
          'role': 'system',
          'content': _systemPrompt(stageId),
        },
        {'role': 'user', 'content': prompt},
      ],
      'temperature': 0.2,
      'stream': false,
    });

    final resp = await _client
        .post(
          Uri.parse('${config.baseUrl}/chat/completions'),
          headers: {
            'Content-Type': 'application/json',
            if (config.apiKey.isNotEmpty) 'Authorization': 'Bearer ${config.apiKey}',
          },
          body: body,
        )
        .timeout(const Duration(seconds: 180));

    if (resp.statusCode != 200) {
      throw Exception('LLM 请求失败: HTTP ${resp.statusCode} ${resp.body}');
    }

    final decoded = jsonDecode(utf8.decode(resp.bodyBytes)) as Map<String, dynamic>;
    final contentStr = (decoded['choices'] as List).first['message']['content'] as String;
    final parsed = _extractJson(contentStr);

    return ChapterAnalysis(
      chapterId: chapterId,
      chapterPath: chapterPath,
      stageId: stageId,
      suggestedStageId: parsed['suggested_stage_id'] as String?,
      tags: ((parsed['tags'] as List?) ?? const []).map((e) => e.toString()).toList(),
      summary: (parsed['summary'] as String?) ?? '',
      splitPoints: ((parsed['split_points'] as List?) ?? const [])
          .map((e) => SplitPoint.fromJson(e as Map<String, dynamic>))
          .toList(),
      scenes: ((parsed['scenes'] as List?) ?? const [])
          .map((e) => Scene.fromJson(e as Map<String, dynamic>))
          .toList(),
      relatedChapters: ((parsed['related_chapters'] as List?) ?? const [])
          .map((e) => e.toString())
          .toList(),
      ignoredSuggestions: previousSuggestions,
      inspirationSplits: ((parsed['inspiration_splits'] as List?) ?? const [])
          .map((e) => InspirationSplit.fromJson(e as Map<String, dynamic>))
          .toList(),
      analyzedAt: DateTime.now(),
      model: config.model,
    );
  }

  /// 从 LLM 回复中提取 JSON（容忍 markdown 代码块包裹）
  Map<String, dynamic> _extractJson(String raw) {
    final trimmed = raw.trim();
    var candidate = trimmed;
    // 去除 ```json ... ``` 包裹
    final fence = RegExp(r'^```(?:json)?\s*(.*?)\s*```$', dotAll: true).firstMatch(trimmed);
    if (fence != null) {
      candidate = fence.group(1)!;
    }
    final start = candidate.indexOf('{');
    final end = candidate.lastIndexOf('}');
    if (start >= 0 && end > start) {
      candidate = candidate.substring(start, end + 1);
    }
    return jsonDecode(candidate) as Map<String, dynamic>;
  }

  String _systemPrompt(String stageId) => stageId == '0_日志'
      ? '''
你是创作灵感分解助手。任务：把自由日志分解为多个灵感片段。
硬性规则：
1. 绝不改写、重写、润色原文的任何字符
2. 每个片段必须引用原文行号范围
3. 片段标题与提炼使用原文中的词汇，不发明原文没有的内容
4. 输出纯 JSON，不要 markdown 代码块，不要任何解释
'''
      : '''
你是文本整理助手。任务：分析文本结构，输出 JSON。
硬性规则：
1. 绝不改写、重写、润色、翻译原文的任何字符
2. 只输出结构信息：阶段归类建议、标签、摘要、拆分点、场景、相似章节
3. 摘要与依据使用原文中的词汇，不发明原文没有的内容
4. 拆分点必须引用原文行号
5. 输出纯 JSON，不要 markdown 代码块，不要任何解释
''';

  String _buildPrompt(String stageId, String content, List<String> previousSuggestions) {
    final ignored = previousSuggestions.isEmpty
        ? '（无）'
        : previousSuggestions.join('；');
    final truncated = content.length > 8000 ? content.substring(0, 8000) : content;

    if (stageId == '0_日志') {
      return '''
已忽略的建议（不要再提出）：$ignored

输出 JSON 结构：
{
  "inspiration_splits": [
    {"title": "片段标题", "start_line": 1, "end_line": 3, "summary": "一句话提炼"}
  ]
}

待分析的日志（行号从1开始）：
$truncated
''';
    }

    return '''
工作流阶段（小说创作）：0_日志（动机心境）/ 1_灵感（源头）/ 2_脚本（素材）/ 3_初稿（成文）/ 4_改稿（定稿）。
已忽略的建议（不要再提出）：$ignored

输出 JSON 结构：
{
  "suggested_stage_id": "阶段ID或null",
  "tags": ["人物/场景/情节线标签"],
  "summary": "3-5句摘要",
  "split_points": [{"line": 行号, "reason": "依据"}],
  "scenes": [{"start_line": 1, "end_line": 30, "characters": ["人物"], "summary": "场景摘要"}],
  "related_chapters": []
}

待分析文本（行号从1开始）：
$truncated
''';
  }

  void close() => _client.close();
}
