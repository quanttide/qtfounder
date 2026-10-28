# `design` 接口

`qtfounder design review [ROOT]`：走查界面交付物，数字号与主按钮，再报命中。
`qtfounder design principles`：打印十二条交互设计评判原则全文。

## review 的参数

| 参数 | 缺省 | 说明 |
|---|---|---|
| `ROOT` | `.` | 要走查的目录或 HTML 文件；目录递归找 `.html`，跳过 `.git`、`node_modules`、`target`、`_build` |
| `--json` | 关 | 输出 JSON |
| `--top <N>` | 15 | 目录模式最多列几个文件 |

## 退出码

| 码 | 含义 |
|---|---|
| 0 | 没命中，或目录里没有 HTML 文件 |
| 1 | 有命中（缺陷与嫌疑都算） |
| 2 | 有文件读不了 |

## 判线

第九条：`font-size` 换算成 px 后低于 12 报缺陷，12–14 报嫌疑。`px` 直接用，`rem` 乘 16，`em` 只计数不换算。
第五条：class 含 `primary` 的 `<button>` 一个文件多于一个报一次嫌疑。

## JSON

```json
{
  "files": [
    {"path":"examples/studio/index.html","fonts":27,"px":0,"rem":27,"em":0,"min_px":10.88,"primaries":5}
  ],
  "findings": [
    {"kind":"font","severity":"error","path":"examples/studio/index.html","line":74,"excerpt":"font-size: 0.72rem","advice":"第九条：更小的中文文字一律视为缺陷；辅助文字 ≥ 12px，正文 ≥ 14px"}
  ]
}
```

`kind` 取 `font`（第九条）与 `primary`（第五条）；`severity` 取 `error`（缺陷）与 `suspect`（嫌疑）。`min_px` 一处可换算声明都没有时为 `null`。

## principles

无参数。正文来自随二进制嵌入的 `design-review` Skill 的 `references/principles.md`；缺了该文件时退回 SKILL.md。Skill 没嵌入时退 2。
