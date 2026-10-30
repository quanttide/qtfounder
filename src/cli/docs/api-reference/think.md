# `qtfounder think` 接口

## 一、命令面

```text
qtfounder think load [--db PATH] [--since YYYY-MM-DD] [--limit N] [--dry-run] [--json]
qtfounder think reflect --stdin [--json] [--model PATH]
qtfounder think reflect --list [--due] [--json]
qtfounder think reflect --id <ref_id> [--json]
qtfounder think reflect --observe <ref_id> --result <一句话> [--emotion 0-100] [--json]
qtfounder think model [--model PATH] [--json]
```

`load` 是诊断，`reflect` 是记录，`model` 只读。`reflect` 的四个入口互斥，给零个或多个都报用法错；`--due` 只与 `--list` 同用，`--result` 与 `--emotion` 只与 `--observe` 同用。

## 二、`load`

读 Hermes 会话库（缺省 `~/.hermes/state.db`，只读），筛掉过短与纯确认的消息，分批送模型判断，按环节汇总。`--limit` 限本轮判多少条，`--batch` 与 `--workers` 控分批与并发，`--dry-run` 只统计候选量不调模型，`--out` 把报告写成 markdown。

怎么判不在代码里，在 `prompt.md`：查找顺序是 `--prompt` → `QTFOUNDER_THINK_PROMPT` → 从当前目录逐级往上找 `src/think/load/prompt.md` → `~/.config/qtfounder/think/prompt.md` → 内置源码树。四道都找不到就报错，不内置兜底。

信号取值 `fatigue`／`friction`／空，环节取值 A 到 E。每条命中带一句原话，截取六到四十字。命中数超过 `--max-hits` 时以退出码 1 结束。

## 三、`reflect` 的输入

整条 JSON 从标准输入读入，字段名与 `--json` 输出一致。

| 字段 | 必填 | 写什么 |
|------|------|--------|
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

认知扭曲与归属、控制的合法取值来自模型文件，不在代码里。`--model` 可临时换一份，缺省按 `QTFOUNDER_THINK_MODEL`、源码树、用户级目录的顺序找。

## 四、`reflect` 的输出

记一条时给 `id`、`at`、`missing`、`next_question` 与 `recurring`。`missing` 按模型的栏位顺序列还没填的栏。`next_question` 照第一处空缺给出那一句。`recurring` 只列出现两次以上的扭曲。

```json
{
  "id": "ref_3WT3",
  "at": "2026-10-24T14:12:07Z",
  "missing": ["balanced"],
  "next_question": "如果朋友遇到同样的事，你会怎么对他说？",
  "recurring": [{"label": "个人化", "count": 2}]
}
```

`--list` 给三段，`due` 是到期的实验，`open` 是栏位没填完的记录，`recurring` 同上。`--due` 时 `open` 为空；`--json` 永远给全量，人读时 `open` 只列最近三条。

`--id` 给记录本身加一个 `missing`。`--observe` 回 `id`、`prediction`、`result` 与可选的 `emotion_after`。

## 五、`model`

输出当前生效的模型与来源。`--json` 给 `source`、`screen`（`signal_words`、`bare_ack`、`min_chars`）、`role`（`domains` 与 `controls`）、`fields`（栏位顺序）、`labels`、`questions`、`distortions`（键与标签）。`[screen]` 是 `load` 的宽筛规则，与认知模型同一份文件。

你指定模型时按这个顺序找：`--model` → `QTFOUNDER_THINK_MODEL` → 从当前目录逐级往上找 `src/think/default-contract.toml` → `~/.config/qtfounder/think/model.toml` → 内置。前两道指向的文件不存在时报错，后面两道找不到就用内置。

## 六、退出码

`think` 没有「有命中就报 1」的语义——记录本身不是缺陷。`load` 在命中数超过 `--max-hits` 时退 1，其余情况成功都是 0。错误一律 2，消息走标准错误，包括找不到 id、记录没有行为实验、模型格式不对。

## 七、错误

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
- 模型文件读不了或格式不对——`模型 <路径> 格式不对：…`

## 八、记在哪

你跑 `reflect`，记录落在 `<数据目录>/think/reflect.json`；数据目录取 `QTFOUNDER_DATA`，其次 `$XDG_DATA_HOME/qtfounder`，再次 `~/.local/share/qtfounder`。`load` 只读会话库，不写文件。
