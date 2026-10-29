# execute：待办的收与抽

`qtfounder execute collect` 把一句「这事儿还没做」收成待办，攒成池子；`qtfounder execute draw` 按优先级抽一件开工。什么时候开口收、待办写成什么形状，判断在随 CLI 发布的 `execute-collect` Skill 里，命令只管落库与抽签。

## 收一件

用户说「先记着」「回头再做」时，把它写成一句能直接开工的话，收进池子：

```bash
qtfounder execute collect "给 qtcloud 补 design review 的 Flutter 解析"
```

```text
已收 tdo_K2M9（优先级 1）：给 qtcloud 补 design review 的 Flutter 解析
池里待抽 1 件
```

一句一件。同文本已在池子里的只收一次，回执里报去重了几件——重复提交是安全的。

批量走 `--stdin`，JSON 形状是 `{"todos":[{"text":"…","priority":2},…]}`；`--json` 给机器读：

```bash
qtfounder execute collect --stdin --json <<'JSON'
{"todos":[
  {"text":"给 qtcloud 补 design review 的 Flutter 解析","priority":2},
  {"text":"发 CLI v0.1.0"}
]}
JSON
```

```json
{
  "collected": [
    {"id":"tdo_K2M9","text":"给 qtcloud 补 design review 的 Flutter 解析","priority":2,"at":"2026-10-29T08:12:42Z"},
    {"id":"tdo_R7XA","text":"发 CLI v0.1.0","priority":1,"at":"2026-10-29T08:12:42Z"}
  ],
  "deduplicated": 0,
  "pending": 2
}
```

## 优先级是抽签权重

`--priority` 越大越容易抽中，缺省 1。0 的签永远抽不中，所以收的时候就拒：

```bash
$ qtfounder execute collect "不重要的事" --priority 0   # 退出码 2
{"error":{"code":"USAGE","message":"优先级从 1 起：0 的签永远抽不中"}}
```

只在用户明说「这个急」时才给 `--priority`。不替用户排优先级——排错了，抽签就永远先抽错的那件。

## 抽一件

要动手却不知道先做哪件时：

```bash
qtfounder execute draw
```

```text
抽中 tdo_K2M9（优先级 2）：给 qtcloud 补 design review 的 Flutter 解析
池里还剩 1 件
```

没有优先级差异就均匀随机，有就按权重抽。抽中的报给用户就开工；抽签的意义是省掉挑选的犹豫，别抽完又问「要不要换一件」。

抽走的进 `drawn` 留痕，不自动放回——抽中就做，不做就手改池子文件（`execute.json`）放回。抽完池子空了再抽，报未找到并提示先去收：

```bash
$ qtfounder execute draw   # 退出码 3
{"error":{"code":"NOT_FOUND","message":"待办池是空的","hint":"先跑 qtfounder execute collect 收几件"}}
```

## 出错时看什么

错误一律是 JSON、写在 stderr，退出码三档：`0` 成功（含去重），`2` 用法错，`3` 池子空时抽签。

常见用法错：位置参数与 `--stdin` 同给、`--priority 0`、待办文本为空（空串或纯空白）、`todos` 为空数组、stdin 不是合法 JSON。逐条对照见[接口参考](../api-reference/execute.md)。

## 记录存在哪

一个文件 `execute.json`：`pending` 是待抽的池子，`drawn` 是抽走的历史。位置与 `product` 的记录同一套——开发构建落代码包里的 `src/cli/data/`，正式构建遵循系统惯例放 `$XDG_DATA_HOME/qtfounder`（未设则 `~/.local/share/qtfounder`）。临时换位置设 `QTFOUNDER_DATA`，整个数据目录跟着走。

记歪了或要删，直接改 `execute.json`；删除不设命令。
