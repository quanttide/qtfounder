# 小说资产数据模型

对应 fiction 仓库的资产结构规则（见 [fiction-asset.md](fiction-asset.md)）：

```
一级文件夹 = 一本小说（Novel）
二级文件夹 = 一个阶段（Stage）
文件编号   = 排序（Sequence）
无编号     = 未排序（sequence 为 null）
```

模型是**仓库结构的镜像**——不发明字段，仓库有什么就建模什么。

## 模型定义

### 资产树（入口）

```dart
/// 小说资产树——fiction 仓库的完整镜像
class FictionAssetTree {
  final List<Novel> novels;   // 一级文件夹列表（职场言情/校园言情/重生言情）
}
```

### 一本小说（一级文件夹）

```dart
/// 一本小说
class Novel {
  final String name;          // 目录名：职场言情
  final List<Stage> stages;   // 阶段列表（按目录顺序）
}
```

### 一个阶段（二级文件夹）

```dart
/// 工作流的一个阶段
class Stage {
  final String name;      // 目录名：1_灵感 / 2_脚本 / 4_改稿（照实，不统一）
  final int order;        // 目录顺序（1/2/3/4）
  final List<FictionFile> files;  // 文件列表（已排序在前，未排序在后）
}
```

### 一个文件（文件）

```dart
/// 小说文件
class FictionFile {
  final String fileName;        // 1_1_咖啡厅重逢.md
  final String path;            // 绝对路径
  final Sequence? sequence;     // 编号解析；null = 未排序（无编号）
  final String title;           // 标题：编号后的部分（"咖啡厅重逢"）；无编号=全名（"偷看睡觉"）
  final int version;            // 同排序位置的不同版本：" 2" 后缀 → 2，无 → 1
}
```

### 排序编号（文件编号 = 排序）

```dart
/// 排序编号
class Sequence {
  final int chapter;   // "1_1" → 1
  final int scene;     // "1_1" → 1
  final int compareTo(Sequence other);  // 按 (chapter, scene) 比较
}
```

## 解析规则

### 编号解析（文件名 → 字段）

```
文件名                 →  sequence    title         version
1_1_咖啡厅重逢.md      →  (1, 1)      咖啡厅重逢      1
6_2_海边散步.md        →  (6, 2)      海边散步        1
7_1_酒吧表白 2.md      →  (7, 1)      酒吧表白        2
11_1_赏雪谈心.md       →  (11, 1)     赏雪谈心        1
偷看睡觉.md            →  null        偷看睡觉        1
```

解析规则：
1. 匹配 `^(\d+)_(\d+)(?:_(.*?))?(?: (\d+))?\.md$`：
   - 前两段数字 → Sequence（chapter, scene）
   - 其后文本（去尾部空格 + 数字）→ title + version（" 2" → version 2）
2. 不匹配编号模式 → sequence = null（未排序），title = 文件名（去 .md）
3. 版本后缀独立于编号：`7_1_酒吧表白 2` 的排序位置是 (7,1)，版本是 2

### 排序规则（阶段内文件）

1. 已排序文件（sequence != null）在前，按 (chapter, scene, version) 升序
2. 未排序文件（sequence == null）在后，按文件名自然序
3. 阶段间按 order（目录顺序）排序

## 关系与约束

```
FictionAssetTree
└── Novel（1..n，目录序）
    └── Stage（1..n，目录序 order）
        └── FictionFile（0..n，已排序→未排序）
```

- 一级文件夹**必为小说**：非目录项（README.md/CHANGELOG.md/myst.yml）不进模型——仓库级文件不属于任何小说
- 二级文件夹**必为阶段**：目录名照实记录，不做语义推断（校园言情用"1_素材/2_提纲"也是阶段）
- 文件名**唯一**（文件系统保证）
- 版本号：同排序位置多版本合法（迭代痕迹），version 区分

## 跨端实现

模型是**约定**（文档即契约），各端自行实现：

| 端 | 实现 | 位置 |
| --- | --- | --- |
| Studio（Dart） | class 定义 | `lib/models/fiction/` |
| Provider（Go） | struct 定义 | `internal/creative/` |

序列化（JSON）字段命名对齐：

```json
{
  "novels": [
    {
      "name": "职场言情",
      "stages": [
        {
          "name": "4_改稿",
          "order": 4,
          "files": [
            {"fileName": "1_1_咖啡厅重逢.md", "sequence": {"chapter": 1, "scene": 1}, "title": "咖啡厅重逢", "version": 1}
          ]
        }
      ]
    }
  ]
}
```

## 加载流程

```
fiction 仓库（文件系统）
  → 读顶层目录（过滤 README.md 等）→ Novel
  → 读各小说子目录 → Stage（目录序 = order）
  → 读各阶段文件 → FictionFile（编号解析 + 排序）
  → FictionAssetTree
```

## 与现有代码的关系

- `creative_repository.dart` 当前返回 `CreativeItem`（name/path/category 扁平列表）——扩展为按 `FictionAssetTree` 结构聚合
- `Sequence` 解析复用 provider 已有逻辑（idOf/titleOf/数值排序），两端各自实现同一规则
- 页面（/assets/fiction）直接消费 `FictionAssetTree`：Novel → 卡片，Stage → 可展开目录，FictionFile → 条目

## 未来扩展（不现在实现）

- 记忆资产（/assets/memory）可复用"资产树"模式：`MemoryAssetTree`（一级=分类目录 roadmap/context/journal）
- 阅读详情按 `path` 定位文件（无需新字段）
