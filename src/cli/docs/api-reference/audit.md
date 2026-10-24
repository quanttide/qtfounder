# `audit` 接口

`qtfounder audit [ROOT...]`：合并 `language` 与 `docs` 的结果，出体检报告。

## 参数

| 参数 | 缺省 | 说明 |
|---|---|---|
| `ROOT...` | `.` | 要扫的目录，可给多个，结果并进一份报告 |
| `--language-rules <PATH>` | 逐级往上找 | 语言契约 |
| `--docs-rules <PATH>` | 逐级往上找 | 文档契约 |
| `--json` | 关 | 输出 JSON |

## 退出码

| 码 | 含义 |
|---|---|
| 0 | 没查出真问题 |
| 1 | 有真问题 |
| 2 | 契约读不了 |

## JSON

```json
{
  "scanned_files": 222,
  "errors": 1180,
  "suspects": 784,
  "categories": 150,
  "top": {
    "kind": "banned",
    "advice": "「判定」→ 判断、看出",
    "count": 53,
    "files": 30
  }
}
```

`top` 是本该先改的那一处，没有问题时为 `null`。`categories` 是归堆后的类数，不等于命中数。

## 稳定性

输出给人读，措辞会变，别拿它做解析。要做机器判断，用 `--json` 里的 `count`、`files`、`errors` 三个数。
