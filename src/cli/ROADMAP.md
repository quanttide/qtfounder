# ROADMAP

量潮创始人 CLI 工具（qtfounder-cli）。

## 目标

把创始人工作流中的高频机械操作固化为 CLI 命令，让"规则先行、低摩擦执行"成为默认路径。

## 当前状态

### 已实现：`health` 子命令

从 journal（memory）与小说（fiction）仓库的 commit 历史提取内容，经 LLM 抽取情绪维度（valence/arousal/warning_signs），输出心理健康画像。

| 命令 | 功能 |
|------|------|
| `health check` | 对比近 N 天现实情绪与创作情绪差距 |
| `health track` | 每周情绪追踪，追加 CSV 记录 |
| `health history` | 查看 CSV 历史与均值 |
| `health profile` | 生成情绪基线（均值/波动/预警阈值）到 profile.yaml |

依赖：git2（读取 commit blob）、quanttide-agent（LLM 抽取）。

---

## 路线图

### 阶段 1：journal 生命周期

| 任务 | 说明 |
|------|------|
| `journal today` | 按固定模板（情绪+进展+计划）创建/追加今日 `YYYY-MM-DD.md`，提交信息用文件名 |
| `journal archive` | 批量迁移过期日记到 archive 子模块，自动处理双仓库提交 |
| `journal stats` | 连续记录天数、缺口检测 |

### 阶段 2：子模块更新

| 任务 | 说明 |
|------|------|
| `submodule update` | 从 `git submodule status` 的 `+` 行自动生成 `chore: update <name> submodule` 提交序列 |
| `submodule check` | 列出引用过期、未初始化的子模块 |

### 阶段 3：发布前检查

| 任务 | 说明 |
|------|------|
| `release check` | 校验所有子模块引用为最新 |
| `release check` | CHANGELOG 头部版本与版本号一致 |
| `release check` | 结构类变更（目录迁移/重命名）触发 major 升格提示 |

### 阶段 4：文档一致性校验

| 任务 | 说明 |
|------|------|
| `docs check` | `.agents/skills/` 实际目录 vs AGENTS.md/CONTRIBUTING.md 索引 diff |

---

## 元目标

以 CLI 为载体，验证"把隐性工作习惯显性化为工具"的方法论：流程类功能封装为 Skill，机械校验类功能实现为 CLI 命令。所有命令的输出保持 CLI 友好（简洁、无 emoji）。
