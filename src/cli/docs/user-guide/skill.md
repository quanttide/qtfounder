# `qtfounder skill` 用户指南

随 CLI 发布的 Skill 只有三条：`meta-classify`、`language-govern`、`think-reflect`。你装了二进制，就同时得到这三份说明书；它们点名的命令都由这个工具提供，接口一变文字就得跟着变，所以一起发。

正文在构建时嵌进二进制，装的时候不读源码树，也不联网。装到系统位置以后，这台机器上的所有项目都能用。

## 一、嵌了哪三条

来源是本包的 `skills/` 目录，改来源用环境变量 `QTFOUNDER_SKILLS_DIR`。目录下有 `SKILL.md` 的子目录算一条，其余文本文件当附属文件一起带走。

其余 Skill（提交、子模块、发版、需求这几条）不跟 CLI 走：它们不调这个工具，放在各自仓里直接改，不必装。

## 二、`list`：先看装到哪一步

```bash
qtfounder skill list
qtfounder skill list --root ../data/context
```

缺省看系统位置 `$HOME/.agents/skills`。每条给三种状态：已是最新、与嵌入版本不一致、未安装。比的是磁盘上的 `SKILL.md` 与二进制里的正文，逐字比，不看时间戳。

## 三、`install`：装一条或全部

```bash
qtfounder skill install                  # 三条都装到系统位置
qtfounder skill install meta-classify    # 只装一条
qtfounder skill install --root <仓>      # 只装进某个仓
qtfounder skill install --copy           # 复制而不是链接
qtfounder skill install --claude         # 同时投递到 .claude/skills
qtfounder skill install --force          # 先清掉同名目录再装
```

你给 `--root` 就装进那个仓，不给就装到系统位置 `$HOME`；装出来的位置是 `<安装位置>/.agents/skills/<id>`，同时写一份 `.agents/.skill-lock.json`，记下来源与内容指纹，供工具之间核对。

系统位置上如果已经有一份别的工具写的锁文件、格式我们认不出来，安装照做，只打一句提示，那份文件原样不动。

链接是默认方式：canonical 只有一份，`.claude/skills` 那边是指过来的。系统不给建链接时自动退回复制，输出会说一句。

`--force` 只在重装时用：它先删掉同名目录，上一版留下、这一版已经没有的附属文件不会残留。

## 四、`check`：只查不装

```bash
qtfounder skill check
```

你想接在 CI 或提交前时用它：只列不是 `current` 的 Skill，全都一致退 `0`，有差异退 `1`。

## 五、装完之后

装出去的是复制件的入口，正文仍以本包的 `skills/` 为准。改了 Skill 就重新发版，再 `install` 一次；`check` 会指出哪条旧了。锁文件是工具之间认这套 Skill 的凭据，不随手改。

主仓不需要装：它自己那些 Skill 就地维护，这三条随着 CLI 装在系统位置，在哪个目录下都找得到。
