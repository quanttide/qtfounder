# `product` 接口

需求与澄清两个入口：名词 `requirement` 只读，动词 `clarify` 是唯一写入路径。业务错误一律 JSON 信封写 stderr；参数解析错由 clap 报，走标准错误与退出码 2。

## 一、命令面

```text
qtfounder product requirement [--id <id>] [--json]
qtfounder product clarify [--task <一句话>] [--stdin] [--revise <clr_id>] [--rule <文本>] [--json]
qtfounder product organize [--json]
qtfounder product clear --id <clr_id> [--id <clr_id>] [--json]
```

## 二、`requirement`

不给 `--id` 时列出全部需求：一件事一行，带澄清次数与重复 ≥2 次的差（规则候选）。没有需求时输出「暂无需求，先跑 `qtfounder product clarify`」。

给 `--id` 时输出一件事的详情：任务、时间，以及每条澄清的原以为、实际与提示词。

`--json` 的形状：

```json
{
  "requirements": [
    {
      "id": "req_01J9",
      "task": "给这段代码写单元测试",
      "at": "2026-10-18T09:00:00Z",
      "clarifications": 2,
      "candidates": [
        {
          "actual_key": "铺开写了十段解释",
          "count": 2,
          "clarification_ids": ["clr_01J9", "clr_02B7"]
        }
      ]
    }
  ]
}
```

详情给 `id`、`task`、`at` 与完整的 `clarifications` 数组；单条澄清含 `id`、`at`、`before`、`actual`、`prompt`。

## 三、`clarify`

不带 `--stdin` 时逐行问两问：你原本以为它知道什么、它实际做了什么。没有可挂的需求时先问任务。

`--task "<一句话>"` 新开一件事；不给则挂到最近一次需求上。

`--stdin` 从标准输入读整条 JSON，字段与输出一致：

```json
{
  "task": "给这段代码写单元测试",
  "before": "它知道我只要三行结论",
  "actual": "铺开写了十段解释",
  "prompt": "先给三行以内的结论"
}
```

`before` 与 `actual` 必填；`prompt` 可省略，省略时按两问拼模板。同一条差（`before` 与 `actual` 相同）重复提交不会新增记录，`deduplicated` 为 `true`。

`--revise <clr_id>` 原地修正一条记录，必须配合 `--stdin` 给新的 `before` 与 `actual`，提示词随之重拼。

`--rule "<文本>"` 把候选固化成一条规则，追加进 `rules.md`，并把本次处理的重复差标为已处理。`source` 给出处理的澄清 id，都处理过后再跑为空数组。

成功输出的 `--json` 形状：

```json
{"id": "clr_01J9", "requirement_id": "req_01J9", "at": "2026-10-18T09:12:03Z", "deduplicated": false, "prompt": "…"}
```

## 四、`organize`

把全部记录按 `actual` 归一化键聚成主题，并给出 `rules.md` 里的规则原文。只读，供归档使用。

`--json` 的形状：

```json
{
  "clusters": [
    {
      "key": "铺开写了十段解释",
      "count": 2,
      "records": [
        {
          "id": "clr_01J9",
          "requirement_id": "req_01J9",
          "task": "给这段代码写单元测试",
          "at": "2026-10-18T09:12:03Z",
          "before": "它知道我只要三行结论",
          "actual": "铺开写了十段解释",
          "prompt": "先给三行以内的结论"
        }
      ]
    }
  ],
  "rules": ["先给三行结论"]
}
```

## 五、`clear`

删除已归档的澄清记录。`--id` 可重复；不给 id 是用法错。清空的需求一并移除，`rules.md` 不动。

`--json` 的形状：

```json
{"removed": ["clr_01J9"], "missing": [], "left": 2}
```

`missing` 是记录库里已经找不到的 id；`left` 是清完后剩余的记录数。

## 六、错误

业务错误给 JSON 信封，`code` 为 `USAGE` 或 `NOT_FOUND`：

| 情况 | 退出码 | 输出 |
|------|--------|------|
| 缺必填字段（`before` / `actual`，空文本同） | 2 | `{"error":{"code":"USAGE","message":"缺少字段 actual"}}` |
| 字段值不是文本 | 2 | `{"error":{"code":"USAGE","message":"字段 actual 必须是文本"}}` |
| `--task` 与 `--rule` 同时给 | 2 | `{"error":{"code":"USAGE","message":"--task 与 --rule 不能同时使用"}}` |
| `--stdin` 的 JSON 解析失败 | 2 | `{"error":{"code":"USAGE","message":"JSON 解析失败：第 1 行"}}` |
| `--rule` 文本为空 | 2 | `{"error":{"code":"USAGE","message":"--rule 需要一段文本"}}` |
| `--revise` 未配合 `--stdin` | 2 | `{"error":{"code":"USAGE","message":"--revise 需要配合 --stdin 给新的 before / actual"}}` |
| `--revise` 与 `--task` 或 `--rule` 同给 | 2 | `{"error":{"code":"USAGE","message":"--task 与 --revise 不能同时使用"}}` |
| `--revise` 的记录 id 不存在 | 3 | `{"error":{"code":"NOT_FOUND","message":"clr_XXXX 不存在","hint":"用 qtfounder product requirement --id <需求 id> 查看"}}` |
| 无 `--task` 且没有可挂的需求 | 2 | `{"error":{"code":"USAGE","message":"没有可挂的需求，缺少 task"}}` |
| 交互中读不到回答（EOF / 空行） | 2 | `{"error":{"code":"USAGE","message":"缺少回答"}}` |
| 数据目录读写失败 | 2 | `{"error":{"code":"USAGE","message":"无法读取 <路径>：…"}}` |

参数解析错（未知参数、缺值、缺子命令）由 clap 直接输出，退出码 2，不带 JSON 信封。

## 七、数据放哪

两个文件：`clarify.json` 存记录，`rules.md` 存规则。开发构建（`cargo run` / `cargo test`）落代码包内的 `data/`；发布构建遵循 XDG，落 `$XDG_DATA_HOME/qtfounder`，未设则 `~/.local/share/qtfounder`。`QTFOUNDER_DATA` 优先级最高，供测试与临时切换用。
