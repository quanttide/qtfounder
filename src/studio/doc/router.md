# Studio 路由规划设计

量潮创始人工作台（qtfounder_studio）的路由规划。

## 当前状态

**无路由**——单页应用：

```
Shell（壳）
  └── CreativeDesk（创作现场，唯一内容页）
```

导航栏只有一个导航项，内容区直接内嵌在壳中，未使用 Navigator。这是起步阶段的合理形态：单页无需路由。

## 规划目标

随 ROADMAP 推进（章节详情、情绪状态、设置），工作台将变为多页面应用。路由规划的目标：

| 目标 | 说明 |
| --- | --- |
| 壳内导航 | 导航栏切换一级页面（资产组/领域组） |
| 详情页跳转 | 一级页面内 push 详情（章节详情） |
| Web 可寻址 | 每个页面有 URL（Web 端刷新/分享可直达） |

## 路由表设计（分组）

### 资产组（数据视角：有什么可以看）

| 路径 | 页面 | 导航图标 | 状态 |
| --- | --- | --- | --- |
| `/fiction` | 小说资产（改稿轨迹/章节） | menu_book | 规划（当前创作现场迁移） |
| `/memory` | 记忆资产（roadmap/context 文档） | memory | 规划 |

### 领域组（职能视角：有什么可以用）

| 路径 | 页面 | 导航图标 | 状态 |
| --- | --- | --- | --- |
| `/create` | 创作职能（写作/改稿工作流） | edit | 规划 |
| `/emotion` | 情绪状态（cli 数据接入） | insights | 规划 |

### 二级路由（一级内 push，带参数）

| 路径 | 页面 | 参数 |
| --- | --- | --- |
| `/fiction/chapters/:id` | 章节详情 | 章节编号（如 1_1） |
| `/memory/:category/:name` | 文档详情 | 分类 + 文档名 |

### 路由与导航栏的对应

```
一级路由 ⇄ 导航栏导航项（资产组/领域组一一对应）
二级路由 → 详情页（导航栏保持当前一级，详情作为内容区覆盖）
```

## 技术选型：GoRouter

| 方案 | 评价 |
| --- | --- |
| Navigator 1.0（命名路由） | 简单，但 Web URL 同步弱 |
| **GoRouter（推荐）** | 声明式路由表 + Web URL 同步 + 参数解析，生态标准 |

选 GoRouter 的理由：
1. **Web 可寻址**：`/fiction/chapters/1_1` 直接对应 URL，Web 端刷新/分享不丢状态
2. **声明式路由表**：路由表即文档（与本文档一一对应）
3. **StatefulShellRoute**：壳（Shell）作为外层，一级路由在壳内切换——正好匹配导航栏分组设计

## 壳与路由的集成

```
GoRouter
  └── StatefulShellRoute（壳：导航栏 + 内容区）
        ├── 资产组
        │   ├── /fiction（小说资产）
        │   └── /memory（记忆资产）
        └── 领域组（职能）
            ├── /create（创作职能）
            └── /emotion（情绪职能）
                  └── 二级路由（/fiction/chapters/:id 等，从一级内 push）
```

- 壳（Shell）变为 `StatefulShellRoute` 的容器：导航栏项（按组）⇄ 一级路由索引
- 详情页在内容区内 push，不改变导航栏选中态
- 与导航栏设计（navigation.md）的分组扩展规则一致：新增资产/领域 = 路由表 + 导航项

## Web 路由行为

| 场景 | 行为 |
| --- | --- |
| 访问 `/fiction` | 小说资产 |
| 访问 `/fiction/chapters/1_1` | 直达章节详情 |
| 刷新 | URL 保持，页面状态恢复（路由驱动） |
| 部署 | 需配置 SPA 回退（OSS 静态托管 index.html 兜底） |

## 演进步骤

1. 引入 GoRouter 依赖，定义分组路由表（一级路由 + 壳）
2. 创作现场迁移：拆分为 /fiction（改稿轨迹）与 /memory（文档矩阵）
3. ROADMAP 目标 2：章节详情页（`/fiction/chapters/:id`）+ 列表点击跳转
4. 后续：/emotion 随 cli 数据接入

## 关联文档

- [navigation.md](navigation.md)：导航栏设计（分组结构与扩展规则）
- [index.md](index.md)：整体设计思路
- [ROADMAP.md](../ROADMAP.md)：功能演进
