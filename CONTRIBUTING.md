# CONTRIBUTING

## src/studio 数据源配置方法

创始人工作台（`src/studio`）以 fiction 和 memory 仓库为数据源。数据源路径通过**环境变量（dart-define）**配置，编译期注入，跨平台一致。

### 1. 配置环境变量

```bash
# 方式一：命令行直接注入
flutter run \
  --dart-define=QTFOUNDER_FICTION_PATH=/absolute/path/to/assets/fiction \
  --dart-define=QTFOUNDER_MEMORY_PATH=/absolute/path/to/assets/memory

# 方式二：构建时注入
flutter build web \
  --dart-define=QTFOUNDER_FICTION_PATH=/absolute/path/to/assets/fiction \
  --dart-define=QTFOUNDER_MEMORY_PATH=/absolute/path/to/assets/memory
```

### 2. 变量说明

| 变量 | 说明 | 未配置默认值 |
|------|------|-------------|
| `QTFOUNDER_FICTION_PATH` | fiction 仓库根（改稿章节来源） | `$HOME/repos/quanttide-founder/assets/fiction`（桌面端） |
| `QTFOUNDER_MEMORY_PATH` | memory 仓库根（roadmap/context 来源） | `$HOME/repos/quanttide-founder/assets/memory`（桌面端） |

### 3. 数据源布局约定

配置的路径指向仓库**根目录**，工作台按固定相对结构读取：

```
{fiction}/
└── 职场言情/
    └── 4_改稿/*.md        → 改稿轨迹（章节列表）

{memory}/
├── roadmap/*.md            → 创作方向
└── context/*.md            → 创作方法
```

### 4. 平台行为

| 平台 | 行为 |
|------|------|
| 桌面端（Linux/macOS/Windows） | 直接读文件系统（dart:io），配置路径即真实数据 |
| Web 端 | 无文件系统访问，使用内置示例数据；如需真实数据，需经 API 服务转发 |
| 测试 | 依赖注入（CreativeDesk 的 loader 参数），不访问真实文件系统 |

### 5. 实现位置

| 文件 | 职责 |
|------|------|
| `src/studio/lib/config.dart` | 环境变量读取与默认路径解析 |
| `src/studio/lib/data/creative_repository.dart` | fiction/memory 数据加载（桌面 IO / Web 回退） |
| `src/studio/lib/main.dart` | CreativeDesk 界面（loader 可注入） |
