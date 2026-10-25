# `qtfounder agent` 用户指南

`agent` 这一组：把 Skill 装到该在的位置，再检查它写得对不对。装的时候只搬运，查的时候只读。

随 CLI 发布的 Skill 有四条：`meta-classify`、`language-govern`、`think-reflect`、`product-clarify`。你装了二进制，就同时得到这四份说明书；前三份点名的命令由本 CLI 提供，`product-clarify` 调 lab 版的 `qtfounder-lab`。接口一变文字就得跟着变，所以一起发。

## 一、嵌了哪四条

来源是本包的 `skills/` 目录，改来源用环境变量 `QTFOUNDER_SKILLS_DIR`。目录下有 `SKILL.md` 的子目录算一条，其余文本文件当附属文件一起带走。

其余 Skill（提交、子模块、发版、需求这几条）不跟 CLI 走：它们不调这个工具，放在各自仓里直接改，不必装。

## 二、`list` 与 `check`：看装到哪一步

```bash
qtfounder agent skill list
qtfounder agent skill list --root ../data/context
qtfounder agent skill check
```

缺省看系统位置 `$HOME/.agents/skills`。每条给三种状态：已是最新、与嵌入版本不一致、未安装。比的是磁盘上的 `SKILL.md` 与二进制里的正文，逐字比，不看时间戳。

`check` 只列不是 `current` 的 Skill。全都一致退 `0`，有差异退 `1`，你想接在 CI 或提交前时用它。

## 三、`install`：装一条或全部

```bash
qtfounder agent skill install                  # 四条都装到系统位置
qtfounder agent skill install meta-classify    # 只装一条
qtfounder agent skill install --root <仓>      # 只装进某个仓
qtfounder agent skill install --copy           # 复制而不是链接
qtfounder agent skill install --claude         # 同时投递到 .claude/skills
qtfounder agent skill install --force          # 先清掉同名目录再装
```

你给 `--root` 就装进那个仓，不给就装到系统位置 `$HOME`；装出来的位置是 `<安装位置>/.agents/skills/<id>`，同时写一份 `.agents/.skill-lock.json`，记下来源与内容指纹，供工具之间核对。

系统位置上如果已经有一份别的工具写的锁文件、格式我们认不出来，安装照做，只打一句提示，那份文件原样不动。

链接是默认方式：canonical 只有一份，`.claude/skills` 那边是指过来的。系统不给建链接时自动退回复制，输出会说一句。

`--force` 只在重装时用：它先删掉同名目录，上一版留下、这一版已经没有的附属文件不会残留。

## 四、`lint`：查名字与写法

Agent 靠两样东西找对 Skill：名字说做什么，说明说什么时候用。这两样写歪了，Skill 再有内容也调不出来。

名字按「范围-动词」查：两段、小写连字符、第二段是动词原形、第一段在登记表里、目录名与 frontmatter 的 `name` 一致。`product-requirement` 这类就是反例：`-ment` 收尾，读起来是东西，不是动作。

说明查两样：写没写触发时机，长度有没有超。正文查一级标题；名字指向命令组的，正文里还要点名那条命令。

```bash
qtfounder agent skill lint .agents/skills
qtfounder agent skill lint            # 从当前目录往下找所有 SKILL.md
qtfounder agent skill lint --json
```

拿本仓 `.agents/skills` 跑一遍，命中的是这一条：

```text
.agents/skills/product-requirement/SKILL.md  L2  [错误] verb  第二段以名词后缀「-ment」收尾：「requirement」
          → 换成动词原形，读名字就知道做什么
```

错误与嫌疑分开：错误是名字对不上或写法不合规，嫌疑是习惯问题，读上下文再定。

## 五、标准还在试

检查规则在 `src/agent/default-contract.toml`：分段数、长度、登记过的范围、名词后缀、触发词，都在里面。`--rules` 换一份，`QTFOUNDER_AGENT_CONTRACT` 也行。

这套逻辑还没进工具箱——名字怎么起还在调，等稳定了再往下沉。

## 六、边界

装出去的是复制件的入口，正文仍以本包的 `skills/` 为准。改了 Skill 就重新发版，再 `install` 一次；`check` 会指出哪条旧了。

文字措辞归 `language`，格式归 `docs`。`lint` 只管 Agent 能不能靠名字和说明找对门。

主仓不需要装：它自己那些 Skill 就地维护，这四条随着 CLI 装在系统位置，在哪个目录下都找得到。
