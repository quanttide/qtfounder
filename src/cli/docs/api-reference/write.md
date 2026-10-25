# `qtfounder write` 接口

## 命令

```bash
qtfounder write [ROOT] [--kind <层>] [--rules <契约>] [--top <N>] [--json]
```

`ROOT` 缺省为当前目录；给文件时只审该文件，不受 `[scope] exclude` 影响。

## 参数

| 参数 | 说明 |
|---|---|
| `ROOT` | 要扫的目录或文件 |
| `--kind` | 只看一层：`empty`、`slogan`、`reference`、`structure` |
| `--rules` | 契约路径；缺省按 `src/write/default-contract.toml` 逐级往上找 |
| `--top` | 文本报告最多列几个文件，缺省 15 |
| `--json` | 输出 JSON |

## 退出码

| 码 | 含义 |
|---|---|
| 0 | 没有命中 |
| 1 | 有命中 |
| 2 | 契约或文件读不了 |

## JSON

```json
{
  "scanned_files": 12,
  "findings": [
    {
      "kind": "slogan",
      "severity": "suspect",
      "path": "docs/guide.md",
      "line": 42,
      "excerpt": "这不是工具，而是制度。",
      "advice": "拆成两句：先说什么不对，再说什么对，各给一句理由"
    }
  ]
}
```

`line` 为 `null` 表示文件级命中（如空话密度、长文没有二级标题）。

## 契约

契约是 TOML，四段与四层一一对应：

- `[empty]`：词表、每千字上限、最小字数、改法；
- `[slogan]`：正则与改法；
- `[reference]`：正则与改法；
- `[structure]`：一级标题要求、允许的一级标题数、标题跳级上限、长文分节门槛。

正则写不对时，`write` 在读契约阶段直接报错，不会静默跳过。
