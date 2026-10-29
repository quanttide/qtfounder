# 数据：标注规则设计

## 目的

studio 的数据页是标注规则的设计工具，不执行标注。设计一条规则（什么算一个标签）、试标验规则、导出任务包给标注端；标注在 lite 做，数据进出都走 CLI `qtfounder data` 同一份 XDG 存储。

分工一句话：studio 设计规则，lite 执行标注，CLI 存取数据，三者只共享存储不共享代码。

## 流程

1. 规则列表页读 `DataStore.resolve()`：`QTFOUNDER_DATA` → `$XDG_DATA_HOME/qtfounder` → `~/.local/share/qtfounder`，与 CLI 的解析顺序一致。空态给引导。
2. 「设计规则」进编辑器：规则名、标签集（每格：机器名、显示名、定义、正例、反例）、批次。
3. 「试标 3 段：验规则」从 journal 抽三段真样本按草稿标签走一遍——试标不落盘，只为验规则本身；跳过率高说明定义没盖住内容。
4. 「保存规则」校验后写 `data/rules/<id>.json`（`rul_` 前缀，带已有 id 覆盖）。
5. 「导出任务包」按批次从 journal 抽段，规则做快照，写 `data/tasks/<id>.json`（`tsk_` 前缀）。

## 模型

```dart
class Rule {
  final String id;              // rul_XXXX
  final String name;
  final RuleSource source;      // kind: journal, split: blank-line
  final List<RuleLabel> labels; // name 机器名 / label 显示名 / define / yes / no
  final int batch;
  final String at;
}
```

校验与 CLI `validate_rule` 同一把尺子：规则名不能为空、至少一个标签、标签机器名与显示名都不能为空、机器名不能重复、批次至少 1。不合格时编辑器顶部出横幅，不落盘。

## journal 从哪来

`JournalRepository` 解析仓库根（优先 `--dart-define=QTFOUNDER_ROOT`，否则向上找同时有 `.gitmodules`、`apps/`、`data/` 的目录），递归扫 `data/journal/**/*.md`，按空行与 `---` 切段（纯标题段丢弃）。它只回答「有哪几段」，不落任何标注数据——试标是内存里的会话，结果由标注端产出。

## 边界

- 标注不在 studio 执行：任务包导出后归 lite，结果由 `qtfounder data result import` 进存储。
- 标签的机器名进结果文件参与统计，改名等于换了统计的分组方式；存储里的旧结果不跟着改。
- 规则的 `source` 当前只有 journal / blank-line 一档，加新数据源要同时动 `JournalRepository` 与 CLI 契约文档。
- 保存只覆盖不删：删规则直接删 `data/rules/<id>.json` 文件。
