# 用户指南

面向 qtfounder 的使用者：如何访问与使用创始人工作台与官网。

## 产品入口

| 产品 | 访问方式 | 说明 |
|------|---------|------|
| 创始人官网（site） | `https://qtfounder.site`（部署后） | 创作现场展示：创作流时间线 / 创作域矩阵 / 改稿轨迹 |
| 创始人工作台（studio） | 本地运行（见下） | 以 fiction + memory 为数据源的工作台 |

## 创始人工作台使用

### 桌面端（推荐）

```bash
cd src/studio
flutter run \
  --dart-define=QTFOUNDER_FICTION_PATH=$HOME/repos/quanttide-founder/assets/fiction \
  --dart-define=QTFOUNDER_MEMORY_PATH=$HOME/repos/quanttide-founder/assets/memory
```

打开后即可看到：
- **改稿轨迹**：fiction 改稿章节列表（职场言情/4_改稿）
- **创作域矩阵**：memory 文档（roadmap 方向 + context 方法）

### Web 端

```bash
cd src/studio && flutter run -d chrome
```

Web 端无文件系统，显示内置示例数据；如需真实数据，需启动 provider API（见 [API 参考](../api-reference/index.md)）并配置接入。

## 数据源说明

- **fiction**：小说创作仓库（职场言情 19 章节等）
- **memory**：创始人记忆仓库（roadmap 方向层 + context 方法层）

数据源布局与配置详见 [开发指南](dev-guide/index.md) 与 [CONTRIBUTING](../../CONTRIBUTING.md)。
