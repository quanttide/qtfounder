# 记忆资产页设计（/assets/memory）

量潮创始人工作台的记忆资产页面（资产职能内部）。**页面结构直接对应 memory 仓库的目录结构**——资产长什么样，页面就长什么样。

## 定位

展示 memory 资产：按仓库现有目录组织，可浏览、可点击阅读。只读——创作活动在 /write。

## 资产结构（数据源：memory 仓库）

memory 仓库结构规则：

```
一级文件夹 = 一类记忆      context / intention / journal / profile / report / roadmap
二级文件夹 = 仅 journal    有 default/（日期日志）
文件命名   = 自由命名      无编号（journal 例外：日期即排序）
```

实际结构：

```
memory/
├── context/    8 篇    ← 方法（如何做）：fiction-adaptation、fiction-plot、work-habits…
├── intention/  1 篇    ← 意图（方向/规划）：qtfounder
├── journal/            ← 事实（时间流水）
│   └── default/        ← 日期日志：2026-07-29.md … 2026-08-15.md
├── profile/    1 篇    ← 画像：老街探店小红书文案
├── report/     4 篇    ← 状态（完成度/差距）：fiction、qtgame-war…
└── roadmap/    5 篇    ← 方向（愿景/选择）：fiction、index、voice-input…
```

**结构要点**：
- 一级 = **记忆类型**（认知科学分类框架）——不同于 fiction 的"一本小说"，memory 是"一类记忆"
- 仅 journal 有二级（default/ 日期日志）；其他类型直接文件
- 文件自由命名（无编号）；journal 例外——**日期即排序**（2026-08-15 自然有序）
- 仓库级文件（AGENTS.md/CHANGELOG.md/README.md）不属于任何记忆类型

## 页面结构（= 仓库结构）

```
记忆资产
├── context/    8 篇（方法）
├── intention/  1 篇（意图）
├── journal/    （事实）
│   └── default/  2026-07-29 … 2026-08-15
├── profile/    1 篇（画像）
├── report/     4 篇（状态）
└── roadmap/    5 篇（方向）
```

页面组件（与小说资产页同模式）：
- **类型卡片**（context/intention/journal/…）——对应一级文件夹
- 展开类型 → 文件列表（journal 多一层 default/）
- 目录展开/收起（像文件浏览器），文件名即条目

## 交互

| 动作 | 行为 |
| --- | --- |
| 点击类型 | 展开/收起该类型文件 |
| 点击文件 | 打开阅读（详情页，只读） |
| 计数 | 类型名旁显示文件数（8 / 1 / 1 / 4 / 5） |

## 边界

- 只读：浏览 + 阅读，无编辑/写作
- 不发明视图：不搞分类统计/时间轴等抽象——目录结构本身就是视图
- 与小说资产页同一模式（fiction-asset.md）：结构即界面

## 数据源

- `{memory}/{context|intention|journal|profile|report|roadmap}/*.md`（journal 多一层 `default/`）
- 实现：读取 memory 顶层目录 → 各类型文件（journal 特殊处理二级）——复用资产树加载模式

## 演进

1. 目录树展示（与 /assets/fiction 同期）
2. 点击文件 → 阅读详情页
