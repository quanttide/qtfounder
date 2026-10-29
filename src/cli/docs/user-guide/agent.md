# `qtfounder agent` 用户指南

`agent` 这一组：把 Skill 装到该在的位置，再检查它写得对不对。装的时候只搬运，查的时候只读。

随 CLI 发布的 Skill 有九条：`design-review`、`docs-format`、`execute-collect`、`meta-classify`、`language-govern`、`think-reflect`、`product-collect`、`product-organize`、`write-express`。你装了二进制，就同时得到这九份说明书；它们点名的命令都由本 CLI 提供。接口一变文字就得跟着变，所以一起发。

## 一、嵌了哪九条

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
qtfounder agent skill install                  # 九条都装到系统位置
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

名字按「范围-动词」查：两段、小写连字符、第二段是动词原形、第一段在登记表里、目录名与 frontmatter 的 `name` 一致。`product-management` 这类就是反例：`-ment` 收尾，读起来是东西，不是动作。

说明查两样：写没写触发时机，长度有没有超。正文查一级标题；名字指向命令组的，正文里还要点名那条命令。

```bash
qtfounder agent skill lint .agents/skills
qtfounder agent skill lint            # 从当前目录往下找所有 SKILL.md
qtfounder agent skill lint --json
```

拿本仓 `.agents/skills` 跑一遍，命中的是这一条：

```text
.agents/skills/product-management/SKILL.md  L2  [错误] verb  第二段以名词后缀「-ment」收尾：「management」
          → 换成动词原形，读名字就知道做什么
```

错误与嫌疑分开：错误是名字对不上或写法不合规，嫌疑是习惯问题，读上下文再定。

## 五、标准还在试

检查规则在 `src/agent/default-contract.toml`：分段数、长度、登记过的范围、名词后缀、触发词，都在里面。`--rules` 换一份，`QTFOUNDER_AGENT_CONTRACT` 也行。

这套逻辑还没进工具箱——名字怎么起还在调，等稳定了再往下沉。

## 六、边界

装出去的是复制件的入口，正文仍以本包的 `skills/` 为准。改了 Skill 就重新发版，再 `install` 一次；`check` 会指出哪条旧了。

文字措辞归 `language`，格式归 `docs`。`lint` 只管 Agent 能不能靠名字和说明找对门。

主仓不需要装：它自己那些 Skill 就地维护，这九条随着 CLI 装在系统位置，在哪个目录下都找得到。

## 七、relation：Skill 与命令的边

```bash
qtfounder agent relation                    # 每条 Skill 引用了哪些命令
qtfounder agent relation meta-classify      # 只看一条
qtfounder agent relation --json
```

当前九条 Skill 的引用：

```text
design-review
  qtfounder design review
  qtfounder design principles
docs-format
  qtfounder docs format
execute-collect
  qtfounder execute
  qtfounder execute collect
  qtfounder execute draw
language-govern
  qtfounder audit
  qtfounder language
  qtfounder docs
meta-classify
  qtfounder meta classify
...
未覆盖的命令组：agent、asset、code、git
```

用途：

- 新增一个命令组后，看有没有 Skill 接住它；`未覆盖` 就是缺口。
- 改 Skill 正文后，确认它引用的命令与实际命令一致。
- 给某个 Skill 补说明时，先看它现在引用了哪些命令，避免重复或漏写。

命令从 Skill 正文里的 `qtfounder <组> <动词>` 读出来，写进散文也算引用；这一层不执行命令，也不校验命令是否存在。覆盖对照的是 Agent 契约的 `cli_scopes`，新增命令组时先把它写进契约。
