---
audience: agent
---

# `qtfounder write` 接口

## 命令

```bash
qtfounder write [ROOT] [--genre <画像>] [--audience <画像>] [--topic <画像>]
                [--kind <层>] [--rules <契约>] [--top <N>] [--json] [--profiles]
```

`ROOT` 缺省为当前目录；给文件时只审该文件，不受 `[scope] exclude` 影响。

## 参数

- `ROOT`：要扫的目录或文件
- `--genre`：体裁画像，缺省按 front matter 或文件名推断
- `--audience`：读者画像，缺省 `default`
- `--topic`：主题画像，缺省 `default`
- `--kind`：只看一层，取 `empty`、`slogan`、`reference`、`structure`、`genre`、`audience`、`topic`
- `--rules`：契约路径，缺省按 `src/write/default-contract.toml` 逐级往上找
- `--top`：文本报告最多列几个文件，缺省 15
- `--json`：输出 JSON
- `--profiles`：列出契约里的体裁、读者与主题，不审文件

画像选择顺序：命令行 → front matter（`genre:`、`audience:`、`topic:`）→ 路径推断 → `default`。体裁只从文件名推断，避免把同目录下别的文章一起判；主题不自动推断。

## 退出码

退出码 `0` 是没有命中，或 `--profiles` 正常输出完毕。`1` 表示有命中。
| 2 | 契约或文件读不了 |

## JSON

```json
{
  "scanned_files": 12,
  "findings": [
    {
      "kind": "genre",
      "severity": "suspect",
      "path": "docs/guide.md",
      "line": null,
      "excerpt": "体裁「roadmap」缺「后置代价」一节",
      "advice": "补一节标题含「后置代价、代价」的内容"
    }
  ]
}
```

`line` 为 `null` 表示文件级命中（缺少某一节、空话密度、长文没有二级标题）。

`kind` 取七类：`empty`、`slogan`、`reference`、`structure`、`genre`、`audience`、`topic`。

## 契约

契约是 TOML，先画像后句子：

- `[genre.*]`：该体裁要有的节、禁区、二级标题数量范围；`required` 是「标题或正文里出现任一词就算有」。
- `[audience.*]`：一段里允许的不同反引号标识数、对读者说话的词、是否硬性要求。
- `[topic.*]`：主题词与是否需要「定义」或「术语」一节。
- `[empty]`、`[slogan]`、`[reference]`、`[structure]`：句子形态检查，所有文档都跑。

正则写不对时，`write` 在读契约阶段直接报错，不会静默跳过。
