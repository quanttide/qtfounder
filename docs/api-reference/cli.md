# CLI 命令参考

qtfounder-cli（`src/cli/`，Rust）——基于 git commit 历史与 LLM 的情绪维度分析工具。

## 安装与运行

```bash
cd src/cli && cargo build --release
./target/release/qtfounder-cli health --help
```

依赖：LLM API key（环境变量，用于情绪提取）。

## 命令

### health check

情绪状态检查（近 N 天）。

```bash
qtfounder-cli health check [--memory PATH] [--fiction PATH] [--days N]
```

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--memory` | `~/docs/memory` | memory 仓库路径 |
| `--fiction` | `~/docs/fiction` | fiction 仓库路径 |
| `--days` | `7` | 检查天数窗口 |

### health track

情绪追踪（输出 CSV）。

```bash
qtfounder-cli health track [--memory PATH] [--fiction PATH] [--csv PATH]
```

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--memory` | `~/docs/memory` | memory 仓库路径 |
| `--fiction` | `~/docs/fiction` | fiction 仓库路径 |
| `--csv` | — | CSV 输出路径 |

### health history

历史情绪查询。

```bash
qtfounder-cli health history [--csv PATH]
```

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--csv` | — | CSV 输入路径 |

### health profile

情绪画像生成。

```bash
qtfounder-cli health profile [--memory PATH] [--fiction PATH] [--output DIR]
```

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--memory` | `~/docs/memory` | memory 仓库路径 |
| `--fiction` | `~/docs/fiction` | fiction 仓库路径 |
| `--output` | `output` | 画像输出目录 |

## 输出结构

情绪提取输出 JSON 字段：

| 字段 | 说明 |
|------|------|
| `dominant_mood` | 主导情绪 |
| `valence` | 效价（正负向） |
| `arousal` | 唤醒度 |
| `warning_signs` | 预警信号列表 |
| `emotional_needs` | 情绪需求列表 |

## 用途

- 创始人工作流：从创作/工作记录中追踪情绪状态（对齐 memory 的"情绪日记"素材）
- 数据源与 provider 一致（fiction + memory）
