# `execute` 接口

待办的收与抽：`collect` 只写，`draw` 写中带读——抽走一件同时留痕。业务错误一律 JSON 信封写 stderr；参数解析错由 clap 报，走标准错误与退出码 2。

## 一、命令面

```text
qtfounder execute collect [<TEXT>...] [--priority <N>] [--stdin] [--json]
qtfounder execute draw [--json]
```

## 二、`collect`

一句一件：位置参数各是一件待办，`--priority` 作用于整批，缺省 1。同文本已在 `pending` 里的跳过，计入 `deduplicated`；`drawn` 里的历史不参与去重。

`--stdin` 从标准输入读整批，与位置参数互斥：

```json
{"todos":[{"text":"发 CLI v0.1.0","priority":2},{"text":"写周报"}]}
```

每件的 `priority` 可省略，省略按 1；文本去首尾空白后为空是用法错，`priority` 为 0 是用法错。

`--json` 的形状：

```json
{
  "collected": [
    {"id":"tdo_01J9","text":"发 CLI v0.1.0","priority":2,"at":"2026-10-29T08:12:42Z"}
  ],
  "deduplicated": 0,
  "pending": 1
}
```

`collected` 是本次实际入池的，`deduplicated` 是跳过的件数，`pending` 是入池后待抽的总数。

## 三、`draw`

不带参数。空池报未找到（退 3）；否则按 `priority` 加权抽签，全部为缺省权重时等价于均匀随机。抽出的移出 `pending`，追加进 `drawn`（多一个 `drawn_at`），不自动放回。

`--json` 的形状：

```json
{
  "id": "tdo_01J9",
  "text": "发 CLI v0.1.0",
  "priority": 2,
  "at": "2026-10-29T08:12:42Z",
  "drawn_at": "2026-10-29T09:00:00Z",
  "pending": 0
}
```

`pending` 是抽走后剩余的待抽数。

## 四、错误

业务错误给 JSON 信封，`code` 为 `USAGE` 或 `NOT_FOUND`：

| 情况 | 退出码 | 输出 |
|------|--------|------|
| 位置参数与 `--stdin` 同给 | 2 | `{"error":{"code":"USAGE","message":"位置参数与 --stdin 只能给一个"}}` |
| 既无位置参数也不给 `--stdin` | 2 | `{"error":{"code":"USAGE","message":"给一句话收一件，批量用 --stdin"}}` |
| `todos` 为空数组 | 2 | `{"error":{"code":"USAGE","message":"一件待办都没有"}}` |
| 待办文本为空（含纯空白） | 2 | `{"error":{"code":"USAGE","message":"待办文本不能为空"}}` |
| `--priority` 或单件的 `priority` 为 0 | 2 | `{"error":{"code":"USAGE","message":"优先级从 1 起：0 的签永远抽不中"}}` |
| stdin 不是合法 JSON | 2 | `{"error":{"code":"USAGE","message":"stdin 不是合法 JSON：…"}}` |
| 空池执行 `draw` | 3 | `{"error":{"code":"NOT_FOUND","message":"待办池是空的","hint":"先跑 qtfounder execute collect 收几件"}}` |
| 池子文件解析失败 | 2 | `{"error":{"code":"USAGE","message":"池子文件解析失败 <路径>：…"}}` |
| 数据目录读写失败 | 2 | `{"error":{"code":"USAGE","message":"无法读取 <路径>：…"}}` |

参数解析错（未知参数、缺子命令）由 clap 直接输出，退出码 2，不带 JSON 信封。

## 五、数据放哪

一个文件 `execute.json`：`pending` 是待抽的池子（`id`、`text`、`priority`、`at`），`drawn` 是抽走的历史（同上加 `drawn_at`）。id 前缀 `tdo`，生成方案与 `product` 的记录 id 相同。

目录选择与 `product` 共用一套：开发构建（`cargo run` / `cargo test`）落代码包内的 `data/`；发布构建遵循 XDG，落 `$XDG_DATA_HOME/qtfounder`，未设则 `~/.local/share/qtfounder`。`QTFOUNDER_DATA` 优先级最高，供测试与临时切换用。
