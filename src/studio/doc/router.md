# Studio 路由规划设计

量潮创始人工作台（qtfounder_studio）的路由规划。

## 当前状态

**无路由**——单页应用（Shell 内职能切换）。起步形态合理。

## 规划目标

| 目标 | 说明 |
| --- | --- |
| 壳内导航 | 导航栏切换一级页面（三个职能） |
| 资产内部导航 | 资产职能内按资产类型切换（小说/记忆） |
| 详情页跳转 | 资产内 push 详情（章节阅读） |
| Web 可寻址 | 每个页面有 URL |

## 路由表（三个职能）

### 一级路由（导航栏职能）

| 路径 | 页面 | 状态 |
| --- | --- | --- |
| `/assets` | 资产职能（聚合入口） | 规划 |
| `/write` | 写作职能（创作工作流） | 规划 |
| `/think` | 思考职能（思绪结构化） | 规划 |

### 二级路由（资产内部）

| 路径 | 页面 | 说明 |
| --- | --- | --- |
| `/assets/fiction` | 小说资产 | 目录结构镜像 |
| `/assets/memory` | 记忆资产 | 目录结构镜像 |

### 三级路由（阅读详情）

| 路径 | 页面 | 参数 |
| --- | --- | --- |
| `/assets/fiction/read` | 章节/文档阅读 | 文件路径或 id |

### 路由与导航栏的对应

```
一级路由 ⇄ 导航栏职能项（一一对应）
二级/三级 → 资产内部导航与阅读（导航栏保持 /assets 选中）
```

## 技术选型：GoRouter

| 方案 | 评价 |
| --- | --- |
| Navigator 1.0（命名路由） | 简单，但 Web URL 同步弱 |
| **GoRouter（推荐）** | 声明式路由表 + Web URL 同步 + 参数解析 |

选 GoRouter 的理由：
1. **Web 可寻址**：`/assets/fiction` 直接对应 URL
2. **声明式路由表**：路由表即文档
3. **StatefulShellRoute**：壳作为外层，一级路由在壳内切换——匹配导航栏设计

## 壳与路由的集成

```
GoRouter
  └── StatefulShellRoute（壳：导航栏 + 内容区）
        ├── /assets（资产职能）
        │   ├── /assets/fiction（小说资产）
        │   └── /assets/memory（记忆资产）
        ├── /write（写作职能）
        └── /think（思考职能）
```

## Web 路由行为

| 场景 | 行为 |
| --- | --- |
| 访问 `/assets/fiction` | 直达小说资产 |
| 刷新 | URL 保持，页面状态恢复 |
| 部署 | 需配置 SPA 回退（index.html 兜底） |

## 演进步骤

1. 引入 GoRouter，定义职能路由表（一级 + 壳）
2. `/assets` + `/assets/fiction`（小说资产目录树，替换 CreativeDesk）
3. `/assets/memory`（记忆资产）
4. `/write`、`/think` 随功能接入

## 关联文档

- [navigation.md](navigation.md)：导航栏设计
- [screens/index.md](screens/index.md)：页面设计
- [screens/fiction-asset.md](screens/fiction-asset.md)：小说资产页
