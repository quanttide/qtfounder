# 资产契约与资产目录

统一资产管理模式：**资产契约**（配置文件承载语义规则）+ **资产目录**（通用引擎按契约构建目录树）。

## 概念

```
资产契约（Asset Contract）      配置文件：描述"这个资产长什么样"
  = 语义规则的声明（层级/命名/排序/忽略）

资产目录（Asset Catalog）       运行时产物：按契约构建的通用目录树
  = 引擎：读契约 → 遍历目录 → 解析命名 → 排序 → 输出树

页面（AssetCatalogPage）        通用页面：按契约渲染任何资产
  = 结构即界面（契约驱动的目录镜像）
```

**核心转变**：语义规则从**代码**（NamingStrategy 类）移入**配置**（契约文件）——新增资产 = 新增契约，零代码改动。

## 资产契约（schema）

### fiction 契约示例

```yaml
# src/studio/assets/contracts/fiction.yaml
asset: fiction
label: 小说
root: fiction
ignore:                       # 仓库级文件（不进目录树）
  - README.md
  - CHANGELOG.md
  - myst.yml
levels:                       # 层级语义（逐级目录）
  - key: novel                # 一级 = 一本小说
    label: 小说
  - key: stage                # 二级 = 一个阶段
    label: 阶段
naming:                       # 文件命名规则（编号 = 排序）
  pattern: '^(\d+)_(\d+)(?:_(.*?))?(?: (\d+))?$'
  title: '$3'                 # 标题 = 编号后文本
  sortKey: ['$1', '$2']       # 排序键 = (chapter, scene)
  version: '$4'               # 版本 = " 2" 后缀
  unsorted: last              # 无编号（不匹配）→ 排序在末尾
```

### memory 契约示例

```yaml
# src/studio/assets/contracts/memory.yaml
asset: memory
label: 记忆
root: memory
ignore:
  - README.md
  - CHANGELOG.md
  - AGENTS.md
levels:
  - key: category             # 一级 = 一类记忆
    label: 类型
  - key: folder               # 二级（可选：仅 journal 有 default/）
    label: 子目录
    optional: true
naming:
  pattern: null               # 自由命名（无编号）
  sortKey: null
  unsorted: natural           # 全部按名称自然序
```

### 契约字段

| 字段 | 说明 |
| --- | --- |
| `asset` | 资产标识（fiction/memory/…） |
| `label` | 展示名（小说/记忆） |
| `root` | 仓库根（相对数据源路径） |
| `ignore` | 仓库级文件列表（不进树） |
| `levels` | 层级语义：逐级目录的 key/label；`optional` = 该级可缺（memory 的二级） |
| `naming.pattern` | 文件命名解析正则；`null` = 自由命名 |
| `naming.title` | 标题提取（正则组引用或 null=全名） |
| `naming.sortKey` | 排序键（正则组）；`null` = 无 |
| `naming.version` | 版本后缀（正则组）；无 = 1 |
| `naming.unsorted` | 未匹配文件的排序位置：`last`（已排序在前）/ `natural`（全部自然序） |

## 资产目录（引擎）

```
AssetCatalog.load(contract, dataSourcePath)
  → 遍历 root（跳过 ignore）
  → 逐级 levels 建节点（optional 级缺省跳过）
  → 文件按 naming.pattern 解析（title/sortKey/version）
  → 按命名规则排序（已排序在前 / 自然序 / 日期）
  → 输出 CatalogTree
```

### 通用结构

```dart
/// 目录节点（契约 levels 的每一级）
class CatalogNode {
  final String name;             // 目录名
  final String label;            // 契约语义（小说/阶段/类型…）
  final List<CatalogNode> children;  // 子目录
  final List<CatalogFile> files;     // 文件
}

/// 文件（契约 naming 解析结果）
class CatalogFile {
  final String name;             // 文件名（含扩展名）
  final String path;             // 绝对路径
  final String title;            // 展示名（解析后）
  final String? sortKey;         // 排序键（编号 "1_1" / 日期 / null）
  final int version;             // 版本（默认 1）
}
```

### 排序规则（引擎内置）

| 策略 | 说明 | 使用 |
| --- | --- | --- |
| `sequence-first` | 有 sortKey 按键升序在前，无键按名称在后 | fiction |
| `natural` | 全部按名称自然序 | memory（非 journal） |
| `date` | 按日期（sortKey=日期字符串，自然序即时间序） | memory journal |

## 页面集成

```
/assets（资产职能页）
└── 契约注册表：列出全部契约（fiction/小说、memory/记忆）→ 点击进入

/assets/:asset（通用资产页 AssetCatalogPage）
└── 按契约渲染：CatalogTree → 节点卡片 → 展开 → 文件条目 → 点击阅读
    - fiction：小说卡片 → 阶段（1_灵感/2_脚本/4_改稿）→ 文件（编号排序）
    - memory：类型卡片 → 文件（自由命名）；journal 多一层子目录
```

- 页面组件一套（目录树 + 阅读跳转），由契约驱动 label 与层级
- 阅读页不变（按 path 定位）

## 收益

| 项 | 效果 |
| --- | --- |
| 新增资产 | 写一个契约 yaml，零代码（页面/引擎/模型通用） |
| 语义规则 | 配置化：改契约即改资产解读（如 fiction 新增命名规则） |
| 模型 | 通用 CatalogNode/CatalogFile 替代 Novel/Stage/MemoryCategory 两套 |
| 文档 | 契约即文档（schema 自描述） |

## 边界

- **契约是消费方视角**：配置文件放 **studio 端**（`src/studio/assets/contracts/`）——fiction/memory 是内容仓库，不被应用配置污染；契约表达"studio 如何解读这个资产"
- **语义不进引擎**：引擎只做通用遍历/解析/排序；"小说/记忆"等语义由契约 label 承载，页面展示
- **后端（provider）**：同一契约可在 Go 端复用（解析 YAML 的库各端自带），或先只服务 studio

## 落点（相对现有文档）

| 现有 | 变化 |
| --- | --- |
| models/fiction.md（Novel/Stage/FictionFile/Sequence） | 重构为"fiction 契约 + 通用 CatalogTree" |
| models/memory.md（MemoryCategory/MemoryFolder） | 同上 |
| screens/fiction-asset.md / memory-asset.md | 页面不变（结构即界面），底层改契约驱动 |
| creative_repository.dart | 加载逻辑改为 AssetCatalog 引擎 |

## 演进

1. 定契约 schema（本文件）+ 两份契约 yaml
2. AssetCatalog 引擎（Dart 端，遍历/解析/排序）
3. /assets 聚合页（契约注册表）+ 通用资产页
4. 迁移 fiction/memory 模型文档为契约驱动
5. provider 端复用（如需）
