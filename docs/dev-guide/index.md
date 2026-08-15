# 开发指南

面向 qtfounder 的**开发者**：模块结构、开发环境搭建、开发与验证命令。贡献规则见 [CONTRIBUTING](../../CONTRIBUTING.md)。

## 文档定位

| 文档 | 回答的问题 |
|------|-----------|
| [CONTRIBUTING](../../CONTRIBUTING.md) | 如何合规贡献（提交约定/验证门禁/分层提交） |
| 本文件 | 如何开发（结构/环境/命令） |

## 模块结构

| 模块 | 技术栈 | 职责 | ROADMAP |
|------|--------|------|---------|
| `src/site/` | React + Vite | 创始人官网（创作现场展示） | [site docs](../../src/site/docs/index.md) |
| `src/studio/` | Flutter | 创始人工作台 | [studio ROADMAP](../../src/studio/ROADMAP.md) |
| `src/provider/` | Go | 创作数据 API | [provider ROADMAP](../../src/provider/ROADMAP.md) |
| `src/cli/` | Rust | CLI 工具（health 情绪分析） | [cli ROADMAP](../../src/cli/ROADMAP.md) |

## 开发环境搭建

### 前置要求

| 工具 | 用途 | 验证 |
|------|------|------|
| Flutter SDK | studio | `flutter --version` |
| Go 1.26+ | provider | `go version` |
| Rust toolchain | cli | `cargo --version` |
| Node.js | site | `node --version` |

### 数据源配置（环境变量）

工作台与 API 读取 fiction/memory 仓库，通过环境变量指定：

```bash
export QTFOUNDER_FICTION_PATH=$HOME/repos/quanttide-founder/assets/fiction
export QTFOUNDER_MEMORY_PATH=$HOME/repos/quanttide-founder/assets/memory
```

| 变量 | 用途 | 默认值（桌面端） |
|------|------|-----------------|
| `QTFOUNDER_FICTION_PATH` | fiction 仓库根 | `~/repos/quanttide-founder/assets/fiction` |
| `QTFOUNDER_MEMORY_PATH` | memory 仓库根 | `~/repos/quanttide-founder/assets/memory` |
| `QTFOUNDER_ADDR` | provider 监听地址 | `:8080` |

数据源布局约定：

```
{fiction}/职场言情/4_改稿/*.md   → 改稿轨迹
{memory}/roadmap/*.md            → 创作方向
{memory}/context/*.md            → 创作方法
```

Studio 桌面端默认路径已内置，可直接运行；其他平台或自定义路径需注入（`--dart-define=QTFOUNDER_FICTION_PATH=...`）。

## 开发命令

```bash
# studio（Flutter）
cd src/studio && flutter analyze && flutter test && flutter build web

# provider（Go）
cd src/provider && go build ./... && go vet ./... && go test ./...

# cli（Rust）
cd src/cli && cargo build && cargo test

# site（React）
cd src/site && npm run build
```

## 数据流

```
fiction + memory（数据源）
    ├── → src/studio（桌面端直接读文件系统）
    ├── → src/provider（API）→ studio Web 端
    └── → src/cli（情绪分析）
```

## 调试提示

- **studio 无数据**：检查 QTFOUNDER_* 路径是否正确（桌面端）或是否启动 provider（Web 端）
- **章节顺序乱**：章节按编号数值排序（1_1 在 10_1 前），见 provider repository.go
- **测试跑不过**：studio 测试用依赖注入（不访问真实文件系统）；provider 测试读真实数据源（路径缺失会 skip）
