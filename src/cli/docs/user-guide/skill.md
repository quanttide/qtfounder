# `qtfounder skill` 用户指南

Skill 是这套工作的说明书，住在主仓 `.agents/skills/`。换个仓要用，不必记住主仓在哪，也不必联网：`qtfounder` 在构建时把正文嵌进了二进制，`skill` 负责把它装出去。

## 一、嵌进来的是什么

来源按顺序取：环境变量 `QTFOUNDER_SKILLS_DIR`，没有就看主仓里的相对路径 `../../../../.agents/skills`。目录下有 `SKILL.md` 的子目录算一条，其余文本文件当附属文件一起带走。

从主仓构建时，九条都在；单独 clone 这个仓构建时清单是空的。空清单不假装装好了，`skill list` 会直接说。

## 二、`list`：先看装到哪一步

```bash
qtfounder skill list
qtfounder skill list --root ../data/context
```

每条给三种状态：已是最新、与嵌入版本不一致、未安装。比的是磁盘上的 `SKILL.md` 与二进制里的正文，逐字比，不看时间戳。

## 三、`install`：装一条或全部

```bash
qtfounder skill install                    # 全部装到当前仓
qtfounder skill install git-commit         # 只装一条
qtfounder skill install --copy             # 复制而不是链接
qtfounder skill install --claude           # 同时投递到 .claude/skills
qtfounder skill install --force            # 先清掉同名目录再装
```

装到你用 `--root` 指的那个仓，canonical 只有一份：`<root>/.agents/skills/<id>`。另外写一份 `.agents/.skill-lock.json`，记下来源与内容指纹，供工具之间核对。`--global` 把这套换到主目录。

默认建链接：canonical 只有一份，`.claude/skills` 那边是指过来的。系统不给建链接时自动退回复制，输出会说一句。

`--force` 只在重装时用：它先删掉同名目录，上一版留下、这一版已经没有的附属文件不会残留。

## 四、`check`：只查不装

```bash
qtfounder skill check
```

只列未装与不一致的。全都一致退 `0`，有差异退 `1`，适合接在 CI 或提交前。

## 五、装完之后

装出去的是复制件的入口，正文仍以主仓为准。改了 Skill 就重新 `install` 一次，`check` 会指出哪条旧了。锁文件是工具之间认这套 Skill 的凭据，不随手改。

换一个仓装，就把 `--root` 指过去；装到主目录用 `--global`，那个目录下的 Skill 对这台机器上的所有项目生效。
