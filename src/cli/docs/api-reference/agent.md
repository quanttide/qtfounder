# `qtfounder agent` 接口

## 一、命令面

```text
qtfounder agent skill [ROOT] [--rules PATH] [--top N] [--json]
```

`ROOT` 缺省是当前目录，往下递归找所有 `SKILL.md`；契约里的 `exclude` 跳过流水、依赖与归档目录。

## 二、查什么

每个 `SKILL.md` 过七道：

- `name`：目录名与 frontmatter 一致；小写连字符；分段数与长度合标准
- `scope`：第一段在契约的 `scopes` 里登记过
- `verb`：第二段不以名词后缀收尾
- `description`：存在、不超长、写了触发条件
- `command`：第一段属于 `cli_scopes` 时，正文里要点名 `qtfounder <范围>`
- `heading`：正文有一级标题
- `unique`：同一个名字只有一份

前六道按文件判，`unique` 跨文件判，报在后出现的那个文件上。

## 三、级别与行号

`name`、`scope`、`verb`、`description`、`command`、`heading`、`unique` 里，除 `scope` 与 `command`、`heading` 外都按错误处理。`scope`、`command`、`heading` 是嫌疑：可能真有正当理由。

行号取 frontmatter 里对应那一行；正文级的问题没有行号，只给文件。

## 四、输出

人读按文件排，每条给路径、行号、级别、类型、说明与改法，末尾给合计。

`--json`：

```json
{
  "scanned": 6,
  "findings": [
    {
      "path": ".agents/skills/product-requirement/SKILL.md",
      "line": 2,
      "severity": "error",
      "kind": "verb",
      "message": "第二段以名词后缀「-ment」收尾：「requirement」",
      "advice": "换成动词原形，读名字就知道做什么"
    }
  ]
}
```

`line` 为 `null` 表示文件级。

## 五、退出码

- `0`：没有命中
- `1`：有命中，错误与嫌疑都算
- `2`：目录不存在、契约读不了或格式不对

## 六、契约

你要换一份标准，按这个顺序找。先看 `--rules`；没有就看环境变量 `QTFOUNDER_AGENT_CONTRACT`。再没有，从当前目录逐级往上找源码树里的 `src/agent/default-contract.toml`；最后看 `~/.config/qtfounder/agent/default-contract.toml`。四道都找不到就报错，不内置兜底。

| 段 | 字段 | 用途 |
|----|------|------|
| `[scope]` | `exclude` | 跳过的目录名 |
| `[name]` | `segments`／`max_chars` | 分段数与名字长度上限 |
| | `scopes` | 登记过的范围 |
| | `cli_scopes` | 其中对应 CLI 命令组的那些 |
| | `noun_suffixes` | 名词化后缀，命中即错 |
| `[description]` | `max_chars`／`trigger_words` | 长度上限与触发词 |
| `[body]` | `require_heading` | 正文要不要一级标题 |

## 七、错误

- 目录不存在——`目录不存在：.agents/skill`
- 契约不存在——`--rules 指的文件不存在：./my-contract.toml`
- 契约格式不对——`契约 <路径> 格式不对：…`
- 分段数写成零——`契约 <路径> 的 segments 不能是 0`
- 扫描失败、文件读不了——`扫描失败：…`／`读不了 <路径>：…`
