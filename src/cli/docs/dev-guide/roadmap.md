# ROADMAP

量潮创始人 CLI 工具（qtfounder-cli）。

## 目标

把创始人工作流中的高频机械操作固化为 CLI 命令，让"规则先行、低摩擦执行"成为默认路径。

## 当前状态

脚手架：入口与结构保留，暂无子命令。原 `health` 情绪分析（check/track/history/profile）已移除——默认数据路径随 `docs/` → `assets/` 迁移失效、无使用痕迹，情绪画像归 memory 五轴档案（`profile/emotions.md`）维护。

新功能接入：`src/` 下新增模块 → `CliCommand` 注册 → 依赖加回 `Cargo.toml`。

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

状态查看（过期/未初始化）已由 `qtcloud-devops code status` 覆盖，不重复实现。

### 阶段 3：发布前检查

| 任务 | 说明 |
|------|------|
| `release check` | 校验所有子模块引用为最新（`release audit` 只查工作区干净，不查引用过期） |
| `release check` | 结构类变更（目录迁移/重命名）触发 major 升格提示 |

版本号格式、CHANGELOG 一致性、标签冲突等其余预检已由 `qtcloud-devops release audit` 覆盖，不重复实现。

### 阶段 4：文档一致性校验

| 任务 | 说明 |
|------|------|
| `docs check` | `.agents/skills/` 实际目录 vs AGENTS.md/CONTRIBUTING.md 索引 diff |

---

## 元目标

以 CLI 为载体，验证"把隐性工作习惯显性化为工具"的方法论：流程类功能封装为 Skill，机械校验类功能实现为 CLI 命令。所有命令的输出保持 CLI 友好（简洁、无 emoji）。
