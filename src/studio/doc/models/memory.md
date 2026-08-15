# 记忆资产数据模型

对应 memory 仓库的资产结构规则（见 [../screens/memory-asset.md](../screens/memory-asset.md)）：

```
一级文件夹 = 一类记忆（MemoryCategory）
二级文件夹 = 仅 journal（default/ 日期日志）
文件命名   = 自由命名（无编号；journal 日期即排序）
```

模型是**仓库结构的镜像**——不发明字段。

## 模型定义

### 资产树（入口）

```dart
/// 记忆资产树——memory 仓库的完整镜像
class MemoryAssetTree {
  final List<MemoryCategory> categories;  // 一级文件夹列表（context/intention/journal/…）
}
```

### 一类记忆（一级文件夹）

```dart
/// 一类记忆（认知科学分类框架中的一类）
class MemoryCategory {
  final String name;           // 目录名：context / journal / roadmap
  final int order;             // 目录顺序
  final List<MemoryFile> files;       // 直接文件（大部分类型）
  final List<MemoryFolder> subFolders; // 二级文件夹（仅 journal 有 default/）
}
```

### 二级文件夹（仅 journal）

```dart
/// 二级文件夹（journal/default/）
class MemoryFolder {
  final String name;           // default
  final List<MemoryFile> files; // 日期日志
}
```

### 一个文件

```dart
/// 记忆文件
class MemoryFile {
  final String fileName;    // fiction-adaptation.md / 2026-08-15.md
  final String path;        // 绝对路径
  final String title;       // 文件名（去 .md）；journal 的日期文件 title = 日期
  final bool isJournal;     // 是否日期日志（journal 类型）
}
```

## 结构规则

```
MemoryAssetTree
└── MemoryCategory（1..n，目录序）
    ├── MemoryFile（0..n，自由命名）
    └── MemoryFolder（0..n，仅 journal 的 default/）
        └── MemoryFile（日期日志，日期即排序）
```

- 一级文件夹**必为记忆类型**：仓库级文件（AGENTS.md/CHANGELOG.md/README.md）不进模型
- 二级文件夹**仅 journal 有**：模型允许（subFolders 为空数组即无），不为其他类型强加
- 文件**自由命名**：无编号解析（区别于 fiction 的 Sequence）——`isJournal` 区分日期日志
- **日期即排序**：journal 文件按日期自然序（2026-08-15 > 2026-08-14），无需解析字段

## 与 fiction 模型的差异

| 维度 | fiction（小说资产） | memory（记忆资产） |
| --- | --- | --- |
| 一级语义 | 一本小说（Novel） | 一类记忆（MemoryCategory） |
| 二级 | 阶段（Stage，各线都有） | 仅 journal 有（MemoryFolder） |
| 文件命名 | 编号 = 排序（Sequence 解析） | 自由命名（无编号） |
| 排序 | 已排序 → 未排序 | 日期（journal）/ 名称（其余） |

共同点：**资产树模式**（Tree → 一级 → 二级 → 文件）、只读、镜像结构。

## 跨端实现

| 端 | 实现 | 位置 |
| --- | --- | --- |
| Studio（Dart） | class 定义 | `lib/models/memory/` |
| Provider（Go） | struct 定义 | `internal/creative/`（或 `internal/memory/`） |

序列化（JSON）：

```json
{
  "categories": [
    {
      "name": "journal",
      "order": 3,
      "files": [],
      "subFolders": [
        {"name": "default", "files": [
          {"fileName": "2026-08-15.md", "title": "2026-08-15", "isJournal": true}
        ]}
      ]
    }
  ]
}
```

## 加载流程

```
memory 仓库（文件系统）
  → 读顶层目录（过滤 AGENTS.md 等）→ MemoryCategory（目录序 = order）
  → 读各类型文件 → MemoryFile（自由命名）
  → journal 特殊：读 default/ 子目录 → MemoryFolder（日期排序）
  → MemoryAssetTree
```

## 与现有代码的关系

- `creative_repository.dart` 扩展：新增 `loadMemoryAssetTree()`（与 fiction 共用目录遍历逻辑）
- 页面（/assets/memory）消费 `MemoryAssetTree`：MemoryCategory → 类型卡片，MemoryFile → 条目
- 阅读详情按 `path` 定位文件

## 未来扩展（不现在实现）

- 统一"资产树"抽象：`AssetTree<T>`（Novel/MemoryCategory 共用遍历与排序骨架）——两端结构规则不同，先各自实现，出现重复再抽象
