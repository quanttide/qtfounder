# 数据标注

## 目的

从 quanttide-founder 仓库的 `data/journal/` 里随机抽一批日志段落，人工贴「事实 / 感受 / 需要 / 行动」标签。参考 Label Studio 的最小可用：一个单位、一组标签、逐条过、结果落盘，不做项目管理与多人协作。

## 流程

1. `JournalRepository.resolve()` 解析仓库根：优先 `--dart-define=QTFOUNDER_ROOT`，否则从当前目录向上找同时有 `.gitmodules`、`apps/`、`data/` 的目录，最多八层。
2. `segments()` 递归扫 `data/journal/**/*.md`，按空行与 `---` 分隔线切段；纯标题、纯分隔线的块丢弃。段 id 是「相对路径#段号」，域取自 `journal/` 下的目录名（default、write、think…）。
3. `DataCubit.load()` 打乱后取前 20 段。
4. 段落流逐段决策：点「事实 / 感受 / 需要 / 行动」贴标签并前进，或点「跳过」。
5. 栈空进保存页：过一遍本批标注（标签、来源、正文前三行），确认后落盘。
6. 完成页给出保存路径与条数，可再抽一批。

## 标签

四个标签取自页面规划里「思考」页的方向（日志四分类），是先定死的一组，暂不做配置界面：

| 标签 | 名字 |
| --- | --- |
| `fact` | 事实 |
| `feeling` | 感受 |
| `need` | 需要 |
| `action` | 行动 |

## 结果格式

`<仓库根>/.annotations/journal.jsonl`，JSON Lines，一行一条，追加不覆盖（与资产抽查的 `.trash/` 同例放仓库根，不进 `data/` 内容区）：

```json
{"id":"data/journal/write/2026-10/2026-10-13.md#2","domain":"write","label":"feeling","text":"我今天写《重生》，感觉找到了感觉。","at":"2026-10-29T08:12:42.000"}
```

## 数据模型

```dart
enum AnnotateLabel { fact, feeling, need, action }

class JournalSegment {
  final String id;    // 相对仓库根的来源#段号
  final String path;  // 绝对文件路径
  final String domain;
  final String text;
}
```

批次状态用 sealed class 收敛为八种：`DataLoading`、`DataReady`、`DataSave`、`DataSaving`、`DataDone`、`DataEmpty`、`DataError`。

## 边界

- 切段只认空行与 `---`，不做语义分段；切错了是后续改进项。
- 保存只追加，没有撤销与编辑；标错了追加一条更正记录或手改 JSONL。
- 标签写死在模型里；要改标签集就改 `AnnotateLabel` 与本文档。
