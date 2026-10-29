# `data` 接口

标注数据的三个入口：`rule` 只读加写（`save` 是唯一写入路径），`task` 只读，`result` 读加导入。业务错误一律 JSON 信封写 stderr；参数解析错由 clap 报，走标准错误与退出码 2。

## 一、命令面

```text
qtfounder data rule list [--json]
qtfounder data rule show <ID> [--json]
qtfounder data rule save --stdin [--json]
qtfounder data task list [--json]
qtfounder data task show <ID> [--json]
qtfounder data result list [--task <ID>] [--json]
qtfounder data result import <FILE> [--task <ID>] [--json]
```

## 二、`rule save`

从 stdin 读整条规则 JSON：

```json
{
  "name": "日志四分类",
  "source": {"kind": "journal", "split": "blank-line"},
  "labels": [{"name": "fact", "label": "事实", "define": "发生了什么", "yes": "正例", "no": "反例"}],
  "batch": 20
}
```

`name`（机器名）与 `label`（显示名）必填；`define`、`yes`、`no` 可省略；`batch` 缺省 20；`id` 缺省时生成 `rul_` 前缀新 id，给已有 id 则覆盖那条，给不存在的 id 报未找到。校验规则：规则名不能为空、至少一个标签、标签的机器名与显示名都不能为空、机器名不能重复、`batch` 至少 1。

`--json` 的形状：

```json
{"id": "rul_01J9", "path": "/home/iguo/.local/share/qtfounder/data/rules/rul_01J9.json", "labels": 1}
```

## 三、`rule list` 与 `rule show`

`list` 一行一条（id、名字、标签数、批次）；`show <ID>` 输出全文，含每个标签的定义与正反例。`--json` 分别为：

```json
{"rules": [{"id": "rul_01J9", "name": "日志四分类", "labels": 4, "batch": 20}]}
```

`show --json` 直接给存储里的规则对象（字段同 `save` 的输入，多 `id` 与 `at`）。

## 四、`task list` 与 `task show`

任务包由 studio 导出，本命令组只读。`list` 一行一个（id、规则 id、单元数、时间）；`show <ID>` 给规则快照与单元清单。

```json
{"tasks": [{"id": "tsk_01J9", "rule_id": "rul_01J9", "units": 20, "at": "2026-10-29T08:12:42Z"}]}
```

## 五、`result import`

从文件读标注端导出的 JSONL，一行一条：

```json
{"task": "tsk_01J9", "unit": "data/journal/write/2026-10/2026-10-13.md#2", "label": "fact", "at": "2026-10-29T09:00:00Z"}
```

行里缺 `task` 时用 `--task` 兜底；两者都没有是用法错。`unit` 与 `label` 必填；`at` 缺省补当前时间。按 `task` 分文件追加进 `data/results/<task>.jsonl`，不覆盖已有行——重复导入会翻倍，去重自己按行比对。

`--json` 的形状：

```json
{"imported": 17, "files": ["/home/iguo/.local/share/qtfounder/data/results/tsk_01J9.jsonl"]}
```

## 六、`result list`

按任务聚合统计：总数与各标签条数（条数降序、同数按名升序）。`--task` 只看一个任务，该任务没有结果文件报未找到。

```json
{"tasks": [{"task": "tsk_01J9", "total": 17, "labels": {"fact": 6, "feeling": 5}}]}
```

## 七、错误

业务错误给 JSON 信封，`code` 为 `USAGE` 或 `NOT_FOUND`：

| 情况 | 退出码 | 输出 |
|------|--------|------|
| `save` 未给 `--stdin` | 2 | `{"error":{"code":"USAGE","message":"用 --stdin 给规则 JSON"}}` |
| stdin 不是合法 JSON | 2 | `{"error":{"code":"USAGE","message":"stdin 不是合法 JSON：…"}}` |
| 规则名为空 / 没有标签 / 机器名重复 | 2 | `{"error":{"code":"USAGE","message":"标签机器名重复：fact"}}` |
| `save` 带了不存在的 id | 3 | `{"error":{"code":"NOT_FOUND","message":"rul_XXXX 不存在","hint":"新建规则不要带 id，改由工具生成"}}` |
| `show` 的 id 不存在 | 3 | `{"error":{"code":"NOT_FOUND","message":"rul_XXXX 不存在","hint":"用 qtfounder data rule list 查看"}}` |
| 导入文件不存在 | 3 | `{"error":{"code":"NOT_FOUND","message":"文件不存在：…","hint":"给标注端导出的 JSONL 路径"}}` |
| 导入行缺 `task` 且无 `--task` | 2 | `{"error":{"code":"USAGE","message":"第 1 行缺 task 字段，或用 --task 指定"}}` |
| 导入文件里一行结果都没有 | 2 | `{"error":{"code":"USAGE","message":"文件里一行结果都没有"}}` |
| `result list --task` 指向无结果的任务 | 3 | `{"error":{"code":"NOT_FOUND","message":"任务 tsk_XXXX 没有结果",…}}` |
| 存储目录读写失败 | 2 | `{"error":{"code":"USAGE","message":"无法读取 <路径>：…"}}` |

参数解析错（未知参数、缺子命令、缺值）由 clap 直接输出，退出码 2，不带 JSON 信封。

## 八、数据放哪

三样都在 XDG 数据目录（与 `product`、`execute` 同一根，各占子目录）：

| 数据 | 位置 | 形状 |
|------|------|------|
| 规则 | `data/rules/<id>.json` | 单个 JSON 对象 |
| 任务包 | `data/tasks/<id>.json` | 单个 JSON 对象（规则快照 + 单元） |
| 结果 | `data/results/<task>.jsonl` | JSON Lines，一行一条，追加不覆盖 |

根目录解析：`QTFOUNDER_DATA` → `$XDG_DATA_HOME/qtfounder` → `~/.local/share/qtfounder`。与 `product`、`execute` 不同，本命令组不区分开发与发布构建——共享存储必须位置稳定，studio 与 lite 都按同一规则找数据。
