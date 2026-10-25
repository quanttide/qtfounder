# `qtfounder skill` 接口

## 一、命令面

```text
qtfounder skill list [--root PATH] [--json]
qtfounder skill install [ID] [--root PATH] [--copy] [--claude] [--force] [--json]
qtfounder skill check [--root PATH] [--json]
```

不给 `--root` 就装到系统位置：`HOME` 拼上 `.agents/skills`。读不到 `HOME` 时报错，提示用 `--root` 指定目标仓。`--claude` 只对 `install` 有意义。

## 二、嵌入清单

清单在构建时生成，运行时不再读源码树。生成规则：来源目录下每个含 `SKILL.md` 的子目录算一条，目录名是 id，正文与附属文本文件一并嵌入；非 UTF-8 的附属文件跳过并给一条构建警告。

来源取 `QTFOUNDER_SKILLS_DIR`，没有就是包内的 `skills/`。目录不存在时清单为空，CLI 仍能构建与运行：`list` 打印一句说明后退 `0`，`check` 退 `2`，`install` 报错退 `2`。

你装好后会得到三条：`language-govern`、`meta-classify`、`think-reflect`。它们调用本 CLI 的命令，接口与文字必须同版本。

## 三、状态

`list` 与 `check` 都用同一种比对：读 `<安装位置>/.agents/skills/<id>/SKILL.md`，与嵌入正文逐字比较。

- `current`：一致
- `stale`：文件在，内容不同
- `missing`：文件不在

`check` 只列不是 `current` 的 Skill；全部一致时退 `0`，否则退 `1`。

## 四、JSON

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

## 五、装到哪

canonical 是 `<安装位置>/.agents/skills/<id>`，`--root` 决定安装位置，缺省 `$HOME`。`--claude` 额外在 `<安装位置>/.claude/skills` 建链接；链接不可用时按复制处理，结果里的 `symlink_failed` 为 `true`。

每次安装都更新 `<安装位置>/.agents/.skill-lock.json`：来源类型记为 `self`，附内容指纹与安装位置。锁文件由 `agent-skills-rs` 维护，字段名是 camelCase。

锁文件是尽力而为：已有文件解析不了（比如别的工具写的格式），安装照做并给一句提示，那份文件不改动。

## 六、退出码

- `list`：`0`，清单为空也是 `0`
- `install`：`0` 全部装上；`2` 有失败、id 不认识或清单为空
- `check`：`0` 全部一致；`1` 有未装或不一致；`2` 清单为空或读不了目标位置

## 七、错误

- 清单为空——`这个二进制没嵌 Skill：构建时 skills/ 是空的，或 QTFOUNDER_SKILLS_DIR 指错了目录`
- id 不认识——`没有嵌入「deploy」，可用：language-govern／meta-classify／think-reflect`
- 读不到 `HOME`——`装到系统位置要读 HOME，但没读到；用 --root 指定目标仓`
- 目录清不掉——`清不掉旧目录 <路径>：…`
- 锁文件写不了——`锁文件写不了：…`
