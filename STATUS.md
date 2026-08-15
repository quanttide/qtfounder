# qtfounder 状态报告

> 更新日期：2026-08-15
> 最新 commit：87f5867

## 项目定位

量潮创始人项目：创始人官网 + 工作台 + 创作数据 API + CLI 工具。核心主题——**创作现场**（创作流时间线 / 创作域矩阵 / 改稿轨迹），数据源为 fiction（小说仓库）与 memory（创始人记忆仓库）。

## 模块状态

| 模块 | 说明 | 版本 | 状态 |
|------|------|------|------|
| `src/site/` | 创始人官网（React + Vite） | v0.1.0-alpha.4 | ✅ 从"作品集"转向"创作现场"（时间线/矩阵/改稿轨迹） |
| `src/studio/` | 创始人工作台（Flutter） | 1.0.0+1 | ✅ 壳 + 数据源（fiction/memory 环境变量配置） |
| `src/provider/` | 创作数据 API（Go） | v0.1.0 | ✅ chapters/memory/health 三 API，真实数据源测试通过 |
| `src/cli/` | CLI 工具（Rust） | v0.1.0 | ✅ health 子命令（check/track/history/profile，git 历史 + LLM 情绪维度） |

## 数据源

| 数据源 | 路径（环境变量） | 内容 |
|--------|-----------------|------|
| fiction | `QTFOUNDER_FICTION_PATH` | 改稿章节（职场言情/4_改稿，19 章） |
| memory | `QTFOUNDER_MEMORY_PATH` | roadmap/（方向）+ context/（方法）文档 |

## 验证状态

| 模块 | 验证 |
|------|------|
| src/studio | flutter analyze 零问题 · flutter test 通过 · flutter build web 成功 |
| src/provider | go build / go vet / go test 全通过（真实数据源） |
| src/cli | cargo build / cargo test 通过 |
| src/site | npm 构建正常（v0.1.0-alpha.4） |

## 数据流

```
fiction + memory（数据源，环境变量配置）
    ├── → src/studio（桌面端直接读文件系统）
    ├── → src/provider（API：/api/chapters /api/memory）→ src/studio Web 端
    └── → src/cli（health 情绪分析）
```

## 最近进展

- 2026-08-15：src/provider Go 初始化（创作数据 API）+ src/studio 数据源配置
- 2026-08-15：src/studio Flutter 初始化（量潮创始人工作台·创作现场）
- 2026-08-14：site v0.1.0-alpha.4（创作现场转向 + CLI health）
