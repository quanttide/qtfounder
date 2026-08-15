# 开发指南

面向 qtfounder 的开发者：模块结构、数据源配置、开发与验证流程。

## 模块结构

| 模块 | 技术栈 | 职责 | ROADMAP |
|------|--------|------|---------|
| `src/site/` | React + Vite | 创始人官网（创作现场展示） | [site docs](../../src/site/docs/index.md) |
| `src/studio/` | Flutter | 创始人工作台 | [studio ROADMAP](../../src/studio/ROADMAP.md) |
| `src/provider/` | Go | 创作数据 API | [provider ROADMAP](../../src/provider/ROADMAP.md) |
| `src/cli/` | Rust | CLI 工具（health 情绪分析） | [cli ROADMAP](../../src/cli/ROADMAP.md) |

## 数据源配置（环境变量）

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

配置方法详见 [CONTRIBUTING](../../CONTRIBUTING.md)（注入方式 / 平台行为 / 实现位置）。

## 开发与验证

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

## 工作流

1. 先读 AGENTS.md（工作原则）与本仓库 STATUS.md（当前状态）
2. 各模块改动在模块内提交推送
3. 父仓库（quanttide-founder）更新子模块指针
4. 重要变更同步 STATUS.md / CHANGELOG.md
