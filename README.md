# qtfounder

量潮创始人项目。

## 结构

```
├── src/
│   ├── site/      → 创始人官网（React + Vite）
│   ├── cli/       → CLI 工具（Rust）
│   └── studio/    → 创始人工作台（Flutter，数据源：fiction + memory）
├── .github/workflows/
└── README.md
```

| 目录 | 说明 | 开发命令 |
|------|------|---------|
| `src/site/` | 创始人官网 | `cd src/site && npm run dev` |
| `src/cli/` | CLI 工具 | `cd src/cli && cargo run` |
| `src/studio/` | 创始人工作台 | `cd src/studio && flutter run` |

## src/studio 数据源配置

工作台以 **fiction**（小说创作仓库）和 **memory**（创始人记忆仓库）为数据源，通过环境变量（dart-define）配置路径。

```bash
cd src/studio

# 桌面端（Linux/macOS/Windows）
flutter run \
  --dart-define=QTFOUNDER_FICTION_PATH=$HOME/repos/quanttide-founder/assets/fiction \
  --dart-define=QTFOUNDER_MEMORY_PATH=$HOME/repos/quanttide-founder/assets/memory

# Web 端（无文件系统，使用内置示例数据；需配合 API 服务读取真实数据）
flutter run -d chrome
```

### 环境变量

| 变量 | 用途 | 默认值 |
|------|------|--------|
| `QTFOUNDER_FICTION_PATH` | fiction 仓库根路径（改稿章节数据源） | `~/repos/quanttide-founder/assets/fiction` |
| `QTFOUNDER_MEMORY_PATH` | memory 仓库根路径（roadmap/context 数据源） | `~/repos/quanttide-founder/assets/memory` |

数据源布局约定（未配置时按默认路径读取）：

```
fiction/职场言情/4_改稿/*.md   → 改稿轨迹
memory/roadmap/*.md            → 创作方向
memory/context/*.md            → 创作方法
```

详细配置方法见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 许可

Apache 2.0
