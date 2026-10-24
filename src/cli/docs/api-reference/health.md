# `qtfounder health reflect` 接口

## 一、命令面

```text
qtfounder health reflect --stdin [--json]
qtfounder health reflect --list [--due] [--json]
qtfounder health reflect --id <ref_id> [--json]
qtfounder health reflect --observe <ref_id> --result <一句话> [--emotion 0-100] [--json]
```

四个入口互斥，给零个或多个都报用法错。`--due` 只与 `--list` 同用，`--result` 与 `--emotion` 只与 `--observe` 同用。

不带子命令的 `qtfounder health` 仍是会话语料分析，行为不变。

## 二、记一条的输入

整条 JSON 从标准输入读入，字段名与 `--json` 输出一致。

| 字段 | 必填 | 写什么 |
|------|------|------|
| `situation` | 是 | 情境，一句话，具体到能判断 |
| `thought` | 是 | 自动思维，用当时的原话 |
| `emotion` | 否 | 情绪名 |
| `intensity` | 否 | 0-100，与 `emotion` 成对 |
| `distortions` | 否 | 认知扭曲，中英文都认 |
| `evidence_for` | 否 | 支持这个念头的事实 |
| `evidence_against` | 否 | 反对这个念头的事实 |
| `balanced` | 否 | 替代思维 |
| `domain` | 否 | `company` 或 `self` |
| `control` | 否 | `scarce` 或 `grasp` |
| `experiment` | 否 | 行为实验，见下 |

`experiment` 里 `prediction` 必填，`review_on` 写 `YYYY-MM-DD`。回填字段 `result` 与 `emotion_after` 只能经 `--observe` 写入。

## 三、输出

记一条时给 `id`、`at`、`missing`、`next_question` 与 `recurring`。`missing` 按填栏顺序列出还空的栏，`next_question` 是照第一处空缺给出的那一句，`recurring` 只列出现两次以上的扭曲。

```json
{
  "id": "ref_3WT3",
  "at": "2026-10-24T14:12:07Z",
  "missing": ["balanced"],
  "next_question": "如果朋友遇到同样的事，你会怎么对他说？",
  "recurring": [{"label": "个人化", "count": 2}]
}
```

`--list` 给三段，`due` 是到期的实验，`open` 是栏位没填完的记录，`recurring` 同上。`--due` 时 `open` 为空。`--json` 永远给全量，人读时 `open` 只列最近三条。

`--id` 给记录本身加一个 `missing`。`--observe` 回 `id`、`prediction`、`result` 与可选的 `emotion_after`。

## 四、退出码

成功一律 0，没有「有记录就报 1」的语义——记录本身不是缺陷。错误一律 2，消息走标准错误，包括找不到 id 和记录没有行为实验。

## 五、错误

- 入口给了零个或多个——`四选一：--stdin（记一条）、--list（看待办与重复）、--id（看一条）、--observe（回填结果）`
- 缺 `situation` 或 `thought`——`缺字段 thought（不能为空）`
- 情绪与强度不成对——`给了 emotion 就要给 intensity（0-100）`
- 强度超过 100——`intensity 是 0-100 的强度`
- 认不出的扭曲——`不认识的认知扭曲「瞎猜」，可用：全或无／灾难化／…`
- 归属或控制取值不认识——`domain 只接受 company／self，收到「me」`
- 日期格式不对——`review_on 要写 YYYY-MM-DD，收到「11月7日」`
- `--result` 为空——`--result 不能为空`
- 记录没有行为实验——`ref_XXXX 没有行为实验，无从回填`
- id 不存在——`找不到反思 ref_XXXX，用 --list 看有哪些记录`
- 标准输入不是 JSON——`标准输入不是合法 JSON：…`

## 六、落点

`<数据目录>/reflect.json`，单文件 JSON。数据目录取 `QTFOUNDER_DATA`，其次 `$XDG_DATA_HOME/qtfounder`，再次 `~/.local/share/qtfounder`。
