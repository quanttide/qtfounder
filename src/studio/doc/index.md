# qtfounder Studio 设计思路

量潮创始人工作台（Flutter）——创作现场的可视化客户端。

## 定位：创作现场

工作台不是"创作管理工具"，而是平台规划中**观察层**的落点：让创作过程本身可见。

三视图的选取依据：

| 视图 | 回答的问题 | 依据 |
| --- | --- | --- |
| 改稿轨迹 | 小说写到哪了 | 章节即情绪单元（1_1 重逢 → 12_3 看星星），轨迹即治疗轨迹 |
| 创作域矩阵 | 方向与方法文档 | memory 的 roadmap（方向层）+ context（方法层）双分类 |
| 创作流时间线 | 各条创作线状态 | 平台意图：**让创作可观测** |

为什么是这三个视图：它们对应创作活动的三个基本问题——**进度、资产、状态**。不是功能罗列，而是让"创作现场"这个概念落到界面。

## 架构：数据源 → 仓库层 → UI

```
fiction + memory（数据源，环境变量配置）
        ↓
creative_repository.dart（仓库层：桌面 IO / Web 回退）
        ↓
CreativeDesk（UI，loader 可注入）
```

### 关键决策 1：环境变量数据源

- 变量：`QTFOUNDER_FICTION_PATH` / `QTFOUNDER_MEMORY_PATH`（dart-define 注入）
- 为什么：数据源是**外部仓库**（fiction/memory 独立演进），工作台只是消费端——路径必须可配置，不能硬编码
- 默认值：桌面端 `~/repos/quanttide-founder/assets/...`（开箱即用），Web 端空（无文件系统）
- 与 provider 共用同一变量名：**单一配置，跨端一致**

### 关键决策 2：桌面直读 vs Web 回退

| 平台 | 方案 | 原因 |
| --- | --- | --- |
| 桌面端 | dart:io 直接读文件系统 | 数据源是本地仓库，直读最简 |
| Web 端 | 内置示例数据 | Web 无文件系统；真实数据经 provider API（ROADMAP 目标 3） |

### 关键决策 3：loader 注入（可测试性）

- `CreativeDesk` 接受 `chaptersLoader` / `memoryLoader` 可选参数
- 为什么：widget test 的 FakeAsync 与真实文件 IO 不兼容（pumpAndSettle 超时）——注入使测试用 fake 数据，不访问真实文件系统
- 这是"测试驱动设计"的实例：测试约束催生的架构决策

## 界面：壳 + 分区卡片

- 壳：80px 品牌侧边栏（"量" + 创作现场图标）——与生态内其他 Studio（econ/delib）一致的设计语言
- 内容：分区卡片（图标 + 标题 + 条目列表）——创作现场三视图的可滚动布局
- 风格：浅色（F1F5F9）+ 紫色主色（4F46E5），与生态统一

## 与平台意图的对应

| 平台意图（intention/qtfounder.md） | Studio 落点 |
| --- | --- |
| 让创作可观测 | 改稿轨迹 + 创作流时间线 |
| 让记忆可积累 | 创作域矩阵（roadmap/context 分类展示） |
| 让状态可感知 | （未来）cli 情绪数据接入 |
| 让表达自然生长 | （未来）章节详情页展示正文 |

## 演进方向

见 [ROADMAP.md](../ROADMAP.md)：章节详情 → Web 端接 provider API → 创作轨迹视图。
