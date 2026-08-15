# 写作职能页设计（/write）

量潮创始人工作台的写作职能页——**AI 原生创作工作流编辑器**：以章节为核心、工作流为辅助、AI 整理层为增强。

核心定位一句话：**AI 产出的是结构（元数据），不是内容（文本）。原文永不被动。**

## 定位

资产页显示结构（有什么），写作页提供创作能力（怎么写）。AI 负责**整理归类**（分类、标签、摘要、结构建议），**不改写原文**——所有 AI 输出都是"关于文本的信息"，而非"新的文本"。

## 设计原则

1. **编辑器优先**：写作体验是核心，其他功能围绕编辑器展开
2. **原文不可变**：AI 在架构上没有改写通道，`content` 只由作者编辑
3. **整理层与内容层分离**：AI 分析结果存独立元数据，与原文文件解耦
4. **建议不自动生效**：所有 AI 建议经显式确认才成为动作，可单个采纳、可忽略
5. **关注点分离**：导航、编辑、预览、整理、状态管理分离为独立模块
6. **实时反馈**：保存状态、字数统计、工作流位置、分析状态实时可见

## 页面结构

```
┌──────────────────────────────────────────────────────────────────┐
│                    工具栏（顶部）                                   │
│ [返回] [章节名] [保存状态] [字数] [预览] [AI整理] [全屏] [设置]     │
├──────────────┬──────────────────────────┬──────────────────────────┤
│  章节导航      │      编辑器主区域          │      右侧面板             │
│  (侧边栏)     │      (原文只读)           │  (预览 / AI 整理 二选一)   │
├──────────────┼──────────────────────────┼──────────────────────────┤
│ - 阶段分组    │ - Markdown 编辑           │ - Markdown 实时渲染       │
│ - 章节列表    │ - 行号显示                │ - AI 整理结果              │
│ - AI 建议徽标 │ - 语法高亮                │   · 阶段归类建议          │
│ - 标签过滤    │ - 自动保存                │   · 标签/主题             │
│ - 搜索       │ - 拆分点/主题 overlay 标注  │   · 场景时间线            │
│ - 当前高亮    │ - 无任何 AI 写入口         │   · 摘要/相似章节          │
└──────────────┴──────────────────────────┴──────────────────────────┘
```

## 核心模块

### 1. 工作流状态栏（顶部）

显示当前章节在工作流中的位置，提供推进操作。

```
[日志] → [灵感] → [脚本] → [初稿] → [改稿]
                ↑ 当前章节位置     ↳ AI 建议位置（虚线）
```

**功能**：
- 显示当前章节所在阶段
- 点击阶段可快速跳转
- 提供推进按钮（移动到下一阶段）
- 显示每个阶段的章节数量
- 显示 AI 阶段归类建议（虚线标记，采纳后变实线）

### 2. 章节导航侧边栏（左侧）

**功能**：
- 按阶段分组的章节列表
- AI 建议徽标：章节旁显示建议归类阶段（如"→改稿"）
- 标签过滤：按 AI 提取的标签筛选章节
- 相似章节标记：关联章节间显示链接标识
- 搜索、收藏、当前高亮

### 3. 编辑器主区域（中央）

**功能**：
- Markdown 编辑、行号、语法高亮、自动保存
- **原文不可变约束**：编辑器是唯一写入口，且只接受作者输入；AI 结果从不写入 `content`
- **只读 overlay 标注**：拆分建议点显示行号虚线标记、段落主题淡色标注——标注渲染在文本层之上，不进入文本缓冲区
- 摘要悬浮：悬停章节标题显示 AI 摘要

### 4. 预览 / AI 整理面板（右侧，二选一）

预览模式保持 Markdown 实时渲染。AI 整理模式展示：

| 区块 | 内容 | 动作 |
|------|------|------|
| 阶段归类建议 | 建议目标阶段 + 依据 | [采纳] [忽略] |
| 标签/主题 | 人物、场景、情节线标签 | [应用] [忽略] |
| 拆分建议 | 行号 + 依据（如"视角切换"） | [采纳]（按边界切文件） |
| 场景时间线 | 场景列表（起止 + 人物在场） | 只读 |
| 章节摘要 | 3-5 句摘要 | 只读 |
| 相似章节 | 重复段落、相近章节 | [对照]（跳转分屏） |

### 5. 工具栏（顶部）

[返回] [章节名称] [保存状态] [字数] [预览] [AI整理] [全屏] [设置]

"AI整理"按钮切换右侧面板模式；"设置"含分析模型选择（本地/远程）与分析触发方式（手动/自动）。

## 数据模型

### 章节模型

```dart
class Chapter {
  final String id;           // 唯一标识
  final String title;        // 章节标题
  final String stageId;      // 所在阶段
  final String path;         // 文件路径
  final DateTime createdAt;  // 创建时间
  final DateTime updatedAt;  // 修改时间
  final int wordCount;       // 字数
  final bool isFavorite;     // 是否收藏
  final int sortOrder;       // 排序顺序
}
```

### 工作流模型

```dart
class Workflow {
  final List<Stage> stages;  // 阶段列表
  final String currentStageId; // 当前阶段
}

class Stage {
  final String id;           // 阶段ID
  final String name;         // 阶段名称
  final String semantics;    // 阶段语义
  final int order;           // 排序顺序
  final List<Chapter> chapters; // 该阶段的章节
}
```

### 整理层模型（新增）

```dart
class ChapterAnalysis {
  final String chapterId;
  final String? suggestedStageId;        // 阶段归类建议
  final List<String> tags;               // 标签（人物/场景/情节线）
  final String summary;                  // 章节摘要（只读）
  final List<SplitPoint> splitPoints;    // 拆分建议（只读 + 可采纳）
  final List<Scene> scenes;              // 场景时间线（只读）
  final List<String> relatedChapters;    // 相似章节（只读）
  final List<String> ignoredSuggestions; // 已忽略建议（负反馈）
  final DateTime analyzedAt;             // 分析时间（可追溯）
  final String model;                    // 生成模型（可追溯）
}

class SplitPoint {
  final int line;                        // 行号（原文锚点）
  final String reason;                   // 依据
}

class Scene {
  final int startLine;                   // 场景起始行
  final int endLine;                     // 场景结束行
  final List<String> characters;         // 在场人物
  final String summary;                  // 场景摘要
}
```

### 编辑器状态

```dart
class EditorState {
  final String? currentChapterId; // 当前章节
  final String content;           // 编辑内容（仅作者写入）
  final bool isDirty;             // 是否有未保存修改
  final SaveStatus saveStatus;    // 保存状态
  final int wordCount;            // 字数
  final int cursorPosition;       // 光标位置
  final bool isPreviewMode;       // 是否预览模式
  final bool isFullScreen;        // 是否全屏
}

enum SaveStatus {
  saved,      // ✓ 已保存
  saving,     // ○ 保存中
  unsaved,    // ● 未保存
  error,      // ✗ 保存失败
}
```

## 交互流程

### 1. 打开写作页面

```
用户点击"写作"导航项
  → 加载工作流数据
  → 加载章节分析索引（.analysis/）
  → 显示最近编辑的章节（或第一章）
  → 编辑器获得焦点
```

### 2. 切换章节

```
用户点击侧边栏的章节
  → 检查当前章节是否有未保存修改
    → 有：提示保存（或自动保存）
  → 加载目标章节内容
  → 加载目标章节分析结果（如有缓存）
  → 更新编辑器内容
  → 更新工作流状态栏与 AI 建议标记
```

### 3. 编辑内容

```
用户在编辑器输入
  → 更新编辑器状态（isDirty = true）
  → 启动自动保存计时器（3秒）
  → 更新字数统计
  → （如果开启预览）更新预览面板
```

### 4. 保存内容

```
自动保存触发（或用户按 Ctrl+S）
  → 显示保存状态：○ 保存中
  → 调用 Repository 保存文件
  → 成功：显示 ✓ 已保存
  → 失败：显示 ✗ 保存失败，提示重试
  → 保存成功后：标记分析结果过期（stale）
```

### 5. AI 整理分析（新增）

```
用户点击"AI整理" → 面板打开
  → 若分析结果过期或不存在：
    → 显示分析中状态
    → AnalyzeBloc 增量分析（仅变更段落）
    → LLM 输出结构化 JSON
    → 写入 .analysis/<chapterId>.json
  → 面板展示整理结果
```

### 6. 采纳建议（新增）

```
用户在面板点击 [采纳]（阶段归类 / 拆分 / 标签）
  → 确认对话框展示将执行的结构动作
  → 执行结构操作（移动/切分/打标——均不改写 content 文本）
  → 刷新工作流与侧边栏
  → 该建议标记为已处理
```

### 7. 忽略建议（新增）

```
用户在面板点击 [忽略]
  → 建议写入 ignoredSuggestions
  → 后续分析不再重复提出
```

### 8. 推进章节

```
用户点击工作流状态栏的"推进"按钮
  → 确认对话框（可选）
  → 移动文件到下一阶段目录
  → 更新章节的 stageId
  → 刷新工作流数据
  → 显示成功提示
```

## 架构设计

### 分层架构

```
┌─────────────────────────────────────────────────┐
│                 展示层（UI）                      │
│  CreateScreen, ChapterNavigator, EditorArea,    │
│  PreviewPanel, AnalysisPanel                    │
├─────────────────────────────────────────────────┤
│                 状态管理层                       │
│  CreatePageState, EditorState, AnalysisState    │
├─────────────────────────────────────────────────┤
│                 业务逻辑层                       │
│  WorkflowManager, ChapterService,               │
│  AnalysisService（增量分析/缓存/负反馈）          │
├─────────────────────────────────────────────────┤
│                 数据访问层                       │
│  ChapterRepository（原文）, FileStorage         │
│  ChapterAnalysisRepository（整理层）, LLMClient  │
└─────────────────────────────────────────────────┘
```

### 组件职责

| 组件 | 职责 | 状态管理 |
|------|------|----------|
| `CreateScreen` | 页面壳，组合子组件 | `CreatePageState` |
| `ChapterNavigator` | 章节导航、搜索、AI 徽标 | 接收数据，发送事件 |
| `EditorArea` | 内容编辑 + 只读 overlay 标注 | `EditorState` |
| `PreviewPanel` | Markdown 预览 | 无状态，接收内容 |
| `AnalysisPanel` | AI 整理结果展示与采纳 | `AnalysisState` |
| `WorkflowBar` | 工作流状态 + AI 归类建议 | 接收数据，发送事件 |

### 数据流

```
用户交互 → 事件 → 状态更新 → UI 更新
                ↓
            业务逻辑
                ↓
      原文写入 → 内容层持久化
      AI 输出 → 整理层持久化（独立）
```

## 状态管理

### CreatePageState

```dart
class CreatePageState {
  final Workflow workflow;           // 工作流数据
  final String? currentChapterId;    // 当前章节ID
  final EditorState editorState;     // 编辑器状态
  final AnalysisState analysisState; // 整理层状态
  final bool isLoading;              // 加载状态
  final String? error;               // 错误信息

  Chapter? get currentChapter => /* ... */;
  List<Chapter> get chaptersInCurrentStage => /* ... */;
}
```

### EditorState

```dart
class EditorState {
  final String content;              // 编辑内容
  final bool isDirty;                // 是否有修改
  final SaveStatus saveStatus;       // 保存状态
  final int wordCount;               // 字数
  final int cursorPosition;          // 光标位置
  final bool isPreviewMode;          // 是否预览模式
  final bool isFullScreen;           // 是否全屏
  final List<Annotation> annotations; // 只读 overlay 标注（拆分点/主题）
}
```

### AnalysisState（新增）

```dart
class AnalysisState {
  final ChapterAnalysis? analysis;   // 当前章节分析结果
  final bool isLoading;              // 分析中
  final bool isStale;                // 内容变更后过期标记
  final String? error;               // 错误信息

  // 计算属性
  bool get hasSuggestions => /* analysis 含未处理建议 */;
}
```

## 编辑器实现

### 全屏编辑器（推荐）

```dart
class FullScreenEditor extends StatefulWidget {
  final String chapterId;
  final String initialContent;
  final List<Annotation> annotations;   // 只读标注，来自分析结果
  final Function(String) onSave;

  @override
  State<FullScreenEditor> createState() => _FullScreenEditorState();
}

class _FullScreenEditorState extends State<FullScreenEditor> {
  late TextEditingController _controller;
  Timer? _autoSaveTimer;
  SaveStatus _saveStatus = SaveStatus.saved;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController(text: widget.initialContent);
    _controller.addListener(_onContentChanged);
  }

  void _onContentChanged() {
    setState(() {
      _saveStatus = SaveStatus.unsaved;
    });
    _scheduleAutoSave();
  }

  void _scheduleAutoSave() {
    _autoSaveTimer?.cancel();
    _autoSaveTimer = Timer(Duration(seconds: 3), _save);
  }

  Future<void> _save() async {
    setState(() {
      _saveStatus = SaveStatus.saving;
    });
    try {
      await widget.onSave(_controller.text);
      setState(() {
        _saveStatus = SaveStatus.saved;
      });
    } catch (e) {
      setState(() {
        _saveStatus = SaveStatus.error;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: Icon(Icons.arrow_back),
          onPressed: () => _confirmExit(),
        ),
        title: _buildTitle(),
        actions: [
          _buildSaveStatusIndicator(),
          _buildWordCount(),
          _buildPreviewToggle(),
          _buildAnalysisToggle(),   // 新增：切换 AI 整理面板
          _buildFullScreenButton(),
        ],
      ),
      body: _buildEditor(),
    );
  }

  Widget _buildEditor() {
    return Stack(
      children: [
        TextField(
          controller: _controller,
          maxLines: null,
          expands: true,
          textAlignVertical: TextAlignVertical.top,
          decoration: InputDecoration(
            border: InputBorder.none,
            hintText: '开始写作...',
          ),
          style: TextStyle(fontSize: 16, height: 1.8),
        ),
        // 只读标注层：拆分点虚线、主题淡色标记
        // 渲染在文本层之上，不进入文本缓冲区
        AnnotationOverlay(annotations: widget.annotations),
      ],
    );
  }
}
```

### Markdown 预览

```dart
class MarkdownPreview extends StatelessWidget {
  final String content;

  @override
  Widget build(BuildContext context) {
    return Markdown(
      data: content,
      selectable: true,
      styleSheet: MarkdownStyleSheet(
        // 自定义样式
      ),
    );
  }
}
```

### AI 整理面板

```dart
class AnalysisPanel extends StatelessWidget {
  final ChapterAnalysis analysis;
  final void Function(String suggestionId) onApply;   // 采纳 → 结构动作
  final void Function(String suggestionId) onIgnore;  // 忽略 → 负反馈

  @override
  Widget build(BuildContext context) {
    return ListView(
      children: [
        _StageSuggestion(analysis, onApply, onIgnore),
        _TagsSection(analysis, onApply, onIgnore),
        _SplitPointsSection(analysis, onApply, onIgnore),
        _ScenesTimeline(analysis),
        _SummarySection(analysis),
        _RelatedChapters(analysis),
      ],
    );
  }
}
```

### 分屏布局

```dart
class SplitView extends StatelessWidget {
  final Widget left;
  final Widget right;
  final double splitRatio;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Expanded(
          flex: (splitRatio * 100).toInt(),
          child: left,
        ),
        VerticalDivider(width: 1),
        Expanded(
          flex: ((1 - splitRatio) * 100).toInt(),
          child: right,
        ),
      ],
    );
  }
}
```

## 与资产页的协作

### 只读 vs 可写

| 页面 | 权限 | 操作 |
|------|------|------|
| 资产页 | 只读 | 浏览、阅读、查看整理层标注 |
| 写作页 | 可写 | 编辑、新建、推进、采纳结构建议 |

### 导航流程

```
资产页（只读）
  → 点击章节
  → 写作页（可写）
  → 编辑完成后返回
```

### 数据同步

- 资产页和写作页共享同一数据源（原文 + 整理层）
- 写作页的修改实时反映到资产页
- 资产页可展示整理层只读信息（标签、摘要、场景），不可修改
- 切换页面时自动刷新数据

## 演进计划

### 第一阶段：基础重构
1. 将编辑器从对话框改为全屏页面
2. 添加侧边栏导航
3. 实现自动保存功能
4. 添加保存状态指示

### 第二阶段：编辑体验增强
1. 实现 Markdown 实时预览
2. 添加分屏布局
3. 实现快捷键支持
4. 添加字数统计

### 第三阶段：工作流优化
1. 优化工作流状态栏交互
2. 添加章节拖拽排序
3. 实现批量操作
4. 添加版本历史

### 第四阶段：AI 原生编辑器（整理归类，不改写）

#### 4.1 整理只读
1. 章节摘要、标签提取、场景时间线（纯展示，无结构动作）
2. 右侧 AI 整理面板与只读 overlay 标注
3. 分析结果缓存与过期标记

#### 4.2 结构建议
1. 阶段归类建议 + 一键采纳（移动章节，不改文本）
2. 章节拆分建议 + 一键采纳（按边界切文件）
3. 建议忽略与负反馈记忆

#### 4.3 关联分析
1. 相似章节检测与对照视图
2. 重复段落识别
3. 增量分析（仅变更段落）与成本控制

### 第五阶段：高级功能
1. 实现协作编辑（可选）
2. 添加导出功能（PDF、HTML）
3. 实现云端同步

## 技术实现

### 状态管理

使用 Bloc 进行状态管理，遵循单向数据流原则。

```dart
// 章节列表 Bloc
class ChapterListBloc extends Bloc<ChapterListEvent, ChapterListState> {
  final ChapterRepository _repository;

  ChapterListBloc({required ChapterRepository repository})
      : _repository = repository,
        super(ChapterListInitial()) {
    on<LoadChapters>(_onLoadChapters);
    on<SelectChapter>(_onSelectChapter);
    on<CreateChapter>(_onCreateChapter);
    on<DeleteChapter>(_onDeleteChapter);
    on<MoveChapter>(_onMoveChapter);
  }
  // 实现同前版（加载/选择/新建/删除/移动）
}

// 编辑器 Bloc
class EditorBloc extends Bloc<EditorEvent, EditorState> {
  final ChapterRepository _repository;
  Timer? _autoSaveTimer;

  EditorBloc({required ChapterRepository repository})
      : _repository = repository,
        super(EditorState.initial()) {
    on<LoadContent>(_onLoadContent);
    on<UpdateContent>(_onUpdateContent);
    on<SaveContent>(_onSaveContent);
    on<AutoSaveTriggered>(_onAutoSave);
  }
  // 实现同前版（加载/更新/自动保存/保存）
}

// 工作流 Bloc
class WorkflowBloc extends Bloc<WorkflowEvent, WorkflowState> {
  final WorkflowRepository _repository;

  WorkflowBloc({required WorkflowRepository repository})
      : _repository = repository,
        super(WorkflowState.initial()) {
    on<LoadWorkflow>(_onLoadWorkflow);
    on<AdvanceChapter>(_onAdvanceChapter);
  }
  // 实现同前版（加载/推进）
}

// 分析 Bloc（新增）
class AnalyzeBloc extends Bloc<AnalyzeEvent, AnalysisState> {
  final ChapterAnalysisRepository _analysisRepository;
  final ChapterRepository _chapterRepository;
  final LLMClient _llm;
  final Set<String> _pendingIds = {};  // 分析中去重

  AnalyzeBloc({required this._analysisRepository, required this._chapterRepository, required this._llm})
      : super(AnalysisState.initial()) {
    on<AnalyzeChapter>(_onAnalyzeChapter);
    on<MarkStale>(_onMarkStale);
    on<ApplySuggestion>(_onApplySuggestion);
    on<IgnoreSuggestion>(_onIgnoreSuggestion);
  }

  Future<void> _onAnalyzeChapter(
    AnalyzeChapter event,
    Emitter<AnalysisState> emit,
  ) async {
    if (_pendingIds.contains(event.chapterId)) return; // 去重
    _pendingIds.add(event.chapterId);
    emit(state.copyWith(isLoading: true));

    try {
      final existing = await _analysisRepository.getAnalysis(event.chapterId);
      final content = await _chapterRepository.getChapterContent(event.chapterId);

      // 增量分析：内容未变且有缓存则直接复用
      if (existing != null && !state.isStale) {
        emit(state.copyWith(analysis: existing, isLoading: false));
        return;
      }

      final analysis = await _llm.analyzeStructure(
        content: content,
        previousSuggestions: existing?.ignoredSuggestions ?? [], // 负反馈
      );
      await _analysisRepository.saveAnalysis(analysis);
      emit(state.copyWith(analysis: analysis, isLoading: false, isStale: false));
    } catch (e) {
      emit(state.copyWith(isLoading: false, error: e.toString()));
    } finally {
      _pendingIds.remove(event.chapterId);
    }
  }

  Future<void> _onApplySuggestion(
    ApplySuggestion event,
    Emitter<AnalysisState> emit,
  ) async {
    // 结构动作：移动/拆分/打标——全部走 ChapterRepository，
    // 不经过 content 写入路径
    await _chapterRepository.applyStructuralAction(event.action);
    add(AnalyzeChapter(chapterId: state.analysis!.chapterId));
  }

  Future<void> _onIgnoreSuggestion(
    IgnoreSuggestion event,
    Emitter<AnalysisState> emit,
  ) async {
    final analysis = state.analysis;
    if (analysis == null) return;
    final updated = analysis.copyWith(
      ignoredSuggestions: [...analysis.ignoredSuggestions, event.suggestionId],
    );
    await _analysisRepository.saveAnalysis(updated);
    emit(state.copyWith(analysis: updated));
  }
}
```

### 数据访问层

```dart
abstract class ChapterRepository {
  Future<List<Chapter>> getChapters();
  Future<Chapter?> getChapter(String id);
  Future<String> getChapterContent(String id);
  Future<void> saveChapter(String id, String content);
  Future<void> createChapter(String stageId, String title);
  Future<void> deleteChapter(String id);
  Future<void> moveChapter(String id, String targetStageId);
  Future<void> splitChapter(String id, int line, String title); // 新增：拆分
}

class FileChapterRepository implements ChapterRepository {
  final String basePath;

  FileChapterRepository({required this.basePath});

  @override
  Future<List<Chapter>> getChapters() async {
    // 读取文件系统，解析目录结构
  }

  @override
  Future<void> saveChapter(String id, String content) async {
    // 保存到文件系统（原文文件，仅作者写入）
  }
}

// 整理层仓库（新增）——与原文文件系统完全分离
abstract class ChapterAnalysisRepository {
  Future<ChapterAnalysis?> getAnalysis(String chapterId);
  Future<void> saveAnalysis(ChapterAnalysis analysis);
  Future<void> deleteAnalysis(String chapterId);
}

class FileAnalysisRepository implements ChapterAnalysisRepository {
  final String basePath;  // .analysis/

  @override
  Future<ChapterAnalysis?> getAnalysis(String chapterId) async {
    // 读取 .analysis/<chapterId>.json，不存在返回 null
  }

  @override
  Future<void> saveAnalysis(ChapterAnalysis analysis) async {
    // 写入 .analysis/<chapterId>.json
  }
}
```

### LLM 结构化输出协议（新增）

```dart
abstract class LLMClient {
  /// 分析文本结构，返回结构化整理结果。
  /// 协议约束：输出纯 JSON，绝不返回改写后的文本。
  Future<ChapterAnalysis> analyzeStructure({
    required String content,
    required List<String> previousSuggestions, // 负反馈抑制
  });
}
```

Prompt 硬约束（示例）：

```
你是文本整理助手。任务：分析文本结构，输出 JSON。
硬性规则：
1. 绝不改写、重写、润色、翻译原文的任何字符
2. 只输出结构信息：阶段归类建议、标签、摘要、拆分点、场景、相似章节
3. 摘要与依据使用原文中的词汇，不发明原文没有的内容
4. 拆分点必须引用原文行号
```

## 测试策略

### 单元测试

- 测试状态管理逻辑（含 AnalyzeBloc：缓存复用、负反馈抑制、去重）
- 测试业务逻辑（推进、新建、拆分、采纳建议）
- 测试数据访问层（原文仓库与整理层仓库隔离）

### 原文不可变测试（新增，验收核心）

- **测试内容层纯净性**：执行任意 AI 建议（采纳/忽略/分析）后，断言 `content` 与操作前逐字符相等
- **测试 git diff 为零**：集成测试中执行完整 AI 流程后，检查原文文件 `git diff` 无变化
- **测试整理层隔离**：删除 `.analysis/` 目录后，原文内容与文件结构完全不受影响

### Widget 测试

- 测试组件渲染（含 AnalysisPanel 各区块）
- 测试用户交互（采纳/忽略/切换面板）
- 测试状态更新

### 集成测试

- 测试完整流程（编辑 → 保存 → 分析 → 采纳 → 推进）
- 测试页面间导航
- 测试数据持久化（原文 + 整理层）

## 总结

新的设计以 AI 原生编辑器为核心：**AI 整理归类，永不改写原文**。关键改进：

1. **编辑器优先**：全屏编辑器替代对话框，提供更好的编辑体验
2. **原文不可变**：AI 在架构上没有改写通道——整理层与内容层物理分离，可测试保证（`git diff` 为零）
3. **整理层独立**：分析结果存 `.analysis/`，删除即回到纯文本时代，零侵入
4. **建议不自动生效**：所有建议显式采纳，可忽略且负反馈抑制重复
5. **模块化设计**：导航、编辑、预览、整理分离，易于维护和扩展
6. **状态管理**：Bloc 单向数据流，分析逻辑可测试、可去重、可缓存
7. **工作流集成**：AI 阶段归类建议融入工作流状态栏，采纳即结构动作
