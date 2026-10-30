# `qtfounder agent` 接口

## 一、命令面

```text
qtfounder agent skill list [--root PATH] [--json]
qtfounder agent skill install [ID] [--root PATH] [--copy] [--claude] [--force] [--json]
qtfounder agent skill check [--root PATH] [--json]
qtfounder agent skill lint [ROOT] [--rules PATH] [--top N] [--json]
```

`list`、`install`、`check` 不给 `--root` 就用系统位置：`HOME` 拼上 `.agents/skills`。读不到 `HOME` 时报错，提示用 `--root` 指定目标仓。`--claude` 只对 `install` 有意义。

`lint` 的 `ROOT` 缺省是当前目录，往下递归找所有 `SKILL.md`；契约里的 `exclude` 跳过流水、依赖与归档目录。

## 二、嵌入清单

清单在构建时生成，运行时不再读源码树。生成规则：来源目录下每个含 `SKILL.md` 的子目录算一条，目录名是 id，正文与附属文本文件一并嵌入；非 UTF-8 的附属文件跳过并给一条构建警告。

来源取 `QTFOUNDER_SKILLS_DIR`，没有就是包内的 `skills/`。目录不存在时清单为空，CLI 仍能构建与运行：`list` 打印一句说明后退 `0`，`check` 退 `2`，`install` 报错退 `2`。

你装好后会得到十一条：`code-reflect`、`code-review`、`design-review`、`docs-format`、`execute-collect`、`language-govern`、`meta-classify`、`product-collect`、`product-organize`、`think-reflect`、`write-express`。它们点名的命令都由本 CLI 提供；接口与文字必须同版本。

## 三、装到哪

canonical 是 `<安装位置>/.agents/skills/<id>`，`--root` 决定安装位置，缺省 `$HOME`。`--claude` 额外在 `<安装位置>/.claude/skills` 建链接；链接不可用时按复制处理，结果里的 `symlink_failed` 为 `true`。

每次安装都更新 `<安装位置>/.agents/.skill-lock.json`：来源类型记为 `self`，附内容指纹与安装位置。锁文件由 `agent-skills-rs` 维护，字段名是 camelCase。

锁文件是尽力而为：已有文件解析不了（比如别的工具写的格式），安装照做并给一句提示，那份文件不改动。

## 四、安装状态

`list` 与 `check` 都用同一种比对：读 `<安装位置>/.agents/skills/<id>/SKILL.md`，与嵌入正文逐字比较。

- `current`：一致
- `stale`：文件在，内容不同
- `missing`：文件不在

`check` 只列不是 `current` 的 Skill；全部一致时退 `0`，否则退 `1`。

## 五、`lint` 查什么

每个 `SKILL.md` 过七道：

- `name`：目录名与 frontmatter 一致；小写连字符；分段数与长度合标准
- `scope`：第一段在契约的 `scopes` 里登记过
- `verb`：第二段不以名词后缀收尾
- `description`：存在、不超长、写了触发条件
- `command`：第一段属于 `cli_scopes` 时，正文里要点名 `qtfounder <范围>`
- `heading`：正文有一级标题
- `unique`：同一个名字只有一份

`scope`、`command`、`heading` 只算嫌疑，其余按错误处理。行号取 frontmatter 里对应那一行；正文级的问题没有行号，只给文件。

## 六、JSON

`list` 与 `check`：

```json
{
  "source": "/home/you/.cargo/registry/.../skills",
  "root": "/home/you",
  "skills": [
    {
      "id": "meta-classify",
      "description": "判断一条内容该进哪一格、要不要跨域……",
      "status": "current",
      "path": "/home/you/.agents/skills/meta-classify/SKILL.md"
    }
  ]
}
```

清单为空时 `source` 为 `null`，另给一个 `note`。

`install`：

```json
{
  "root": "/home/you",
  "installed": [
    {
      "id": "meta-classify",
      "path": "/home/you/.agents/skills/meta-classify",
      "symlink_failed": false
    }
  ],
  "failed": []
}
```

`failed` 每条的 `error` 是纯文本，一条失败不影响同批其他条。

`lint`：

```json
{
  "scanned": 6,
  "findings": [
    {
      "path": ".agents/skills/product-management/SKILL.md",
      "line": 2,
      "severity": "error",
      "kind": "verb",
      "message": "第二段以名词后缀「-ment」收尾：「management」",
      "advice": "换成动词原形，读名字就知道做什么"
    }
  ]
}
```

`line` 为 `null` 表示文件级。

## 七、退出码

- `list`：`0`，清单为空也是 `0`
- `install`：`0` 全部装上；`2` 有失败、id 不认识或清单为空
- `check`：`0` 全部一致；`1` 有未装或不一致；`2` 清单为空或读不了目标位置
- `lint`：`0` 没有命中；`1` 有命中；`2` 目录不存在、契约读不了或格式不对

## 八、契约

`lint` 的规则在契约里。你要换一份，按这个顺序找。先看 `--rules`；没有就看环境变量 `QTFOUNDER_AGENT_CONTRACT`。再没有，从当前目录逐级往上找源码树里的 `src/agent/default-contract.toml`；最后看 `~/.config/qtfounder/agent/default-contract.toml`。四道都找不到就报错，不内置兜底。

| 段 | 字段 | 用途 |
|----|------|------|
| `[scope]` | `exclude` | 跳过的目录名 |
| `[name]` | `segments`／`max_chars` | 分段数与名字长度上限 |
| | `scopes` | 登记过的范围 |
| | `cli_scopes` | 其中对应 CLI 命令组的那些 |
| | `noun_suffixes` | 名词化后缀，命中即错 |
| `[description]` | `max_chars`／`trigger_words` | 长度上限与触发词 |
| `[body]` | `require_heading` | 正文要不要一级标题 |

## 九、错误

- 清单为空——`这个二进制没嵌 Skill：构建时 skills/ 是空的，或 QTFOUNDER_SKILLS_DIR 指错了目录`
- id 不认识——`没有嵌入「deploy」，可用：code-reflect／code-review／…／write-express`（十一个 id 按字母序）
- 读不到 `HOME`——`装到系统位置要读 HOME，但没读到；用 --root 指定目标仓`
- 目录清不掉——`清不掉旧目录 <路径>：…`
- 锁文件写不了——提示一句，安装继续
- `lint` 的目录不存在——`目录不存在：.agents/skill`
- `lint` 的契约不存在——`--rules 指的文件不存在：./my-contract.toml`
- 契约格式不对——`契约 <路径> 格式不对：…`
- 分段数写成零——`契约 <路径> 的 segments 不能是 0`
