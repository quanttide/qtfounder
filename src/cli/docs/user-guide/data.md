# data：标注规则、任务包与结果

标注这件事拆三样东西：规则说清「什么算一个标签」，任务包是给标注端的输入（规则快照 + 待标单元），结果是标注端干完活导出的 JSONL。`qtfounder data` 管这三样的存取；规则的设计界面在 studio，标注的执行界面在 lite，CLI 这边是同一份数据的脚本入口。

三样都存在 XDG 数据目录下（`~/.local/share/qtfounder/data/`，`QTFOUNDER_DATA` 可换），studio 写进去的规则这里直接看得到，不需要同步动作。

## 一条规则长什么样

```bash
qtfounder data rule save --stdin <<'JSON'
{
  "name": "日志四分类",
  "source": {"kind": "journal", "split": "blank-line"},
  "labels": [
    {"name": "fact", "label": "事实", "define": "发生了什么，可核对", "yes": "今天写了《重生》", "no": "感觉找到了感觉"}
  ],
  "batch": 20
}
JSON
```

```text
已保存 rul_K2M9（1 个标签）：日志四分类
/home/iguo/.local/share/qtfounder/data/rules/rul_K2M9.json
```

- `name` 机器名（`fact`）进结果文件参与统计，`label` 显示名（「事实」）上界面；
- `define` / `yes` / `no` 是设计时把判断标准说清楚的三格：一句话定义、一个正例、一个反例；
- `source` 说标注对象，当前只有 `journal` / `blank-line`（空行与 `---` 切段）一档；
- `batch` 是任务包一次给标注端多少段；
- 不带 `id` 生成新规则（`rul_` 前缀），带已有 `id` 则覆盖那条。

## 看与改

```bash
qtfounder data rule list          # 一行一条：id、名字、标签数、批次
qtfounder data rule show rul_K2M9 # 全文：来源、切段、每个标签的定义与正反例
```

改一条规则就 `show --json` 拿全文，改完带 `id` 再 `save --stdin`。校验与 studio 同一把尺子：规则名不能为空、至少一个标签、机器名不能重复、批次至少 1。

## 任务包与结果

任务包由 studio 导出（规则快照 + 抽好的待标单元），标注端只认它：

```bash
qtfounder data task list              # id、规则、单元数、时间
qtfounder data task show tsk_K2M9     # 规则快照 + 单元清单
```

标注端干完活导出 JSONL（一行一条：`task`、`unit`、`label`），导回来与统计：

```bash
qtfounder data result import lite-export.jsonl   # 按行里的 task 归档，追加不覆盖
qtfounder data result list                        # 按任务统计各标签条数
qtfounder data result list --task tsk_K2M9        # 只看一个任务
```

```text
tsk_K2M9  共 17 条：fact 6、feeling 5、need 4、action 2
```

跳过率高、某标签一条没有，都是规则没设计好的信号——回 studio 改定义或切法再试。

## 出错时看什么

错误一律是 JSON、写在 stderr，退出码三档：`0` 成功，`2` 用法错（校验不过、stdin 不是合法 JSON、缺 task 字段），`3` 没找到（`show` 的 id 不存在、`--task` 指向的任务没结果）。

```bash
$ qtfounder data rule show rul_没有  # 退出码 3
{"error":{"code":"NOT_FOUND","message":"rul_没有 不存在","hint":"用 qtfounder data rule list 查看"}}
```

常见用法错：`save` 忘了 `--stdin`、带了不存在的 `id`（新建不带 id）、导入文件里的行缺 `task` 又没给 `--task`。逐条对照见[接口参考](../api-reference/data.md)。

## 存储位置

三样都在 XDG 数据目录：`data/rules/<id>.json`、`data/tasks/<id>.json`、`data/results/<task>.jsonl`（JSON Lines，一行一条，追加不覆盖）。本命令组不区分开发与发布构建，一律走 XDG——共享存储必须位置稳定；`QTFOUNDER_DATA` 优先级最高，供测试与临时切换用。

与 `product`、`execute` 的数据同一目录根，各占一个子目录，互不干扰。
