# `qtfounder agent skill` 用户指南

Agent 靠两样东西找对 Skill：名字说做什么，说明说什么时候用。这两样写歪了，Skill 再有内容也调不出来。这个命令查的就是这两样，加正文有没有一级标题、有没有点名它对应的命令。

## 一、名字：范围-动词

两段，小写连字符。第一段是命令入口或业务域，第二段是动词原形：

```text
meta-classify      qtfounder meta classify
think-reflect      qtfounder think reflect
language-govern    qtfounder language
git-commit         不是命令组，是业务域
```

三条规矩：第二段别用名词化后缀（`-ance`、`-ment`、`-tion`、`-ing` 这些）；第一段要在契约里登记过；目录名与 frontmatter 的 `name` 必须一致。

`product-requirement` 这类就是典型反例：`-ment` 收尾，读起来是东西，不是动作。

## 二、说明：写清什么时候用

`description` 要有一句触发条件，让 Agent 判得出该不该调。写成「当……时使用」最省事，写成「适用于……」也可以。上限一千零二十四字，超了客户端会截断。

## 三、正文

正文先要有一级标题。名字指向命令组的，正文里还要点名那条命令：`meta-classify` 却不提 `qtfounder meta classify`，说明名与实对不上。

## 四、跑一遍

```bash
qtfounder agent skill .agents/skills   # 查一个目录
qtfounder agent skill                  # 从当前目录往下找所有 SKILL.md
qtfounder agent skill --json           # 给机器看
```

拿本仓 `.agents/skills` 跑一遍，命中的是这一条：

```text
.agents/skills/product-requirement/SKILL.md  L2  [错误] verb  第二段以名词后缀「-ment」收尾：「requirement」
          → 换成动词原形，读名字就知道做什么
```

错误与嫌疑分开：错误是名字对不上或写法不合规，嫌疑是习惯问题，读上下文再定。

## 五、标准还在试

规则在 `src/agent/default-contract.toml`：分段数、长度、登记过的范围、名词后缀、触发词，都在里面。`--rules` 换一份，`QTFOUNDER_AGENT_CONTRACT` 也行。

这套逻辑还没进工具箱——名字怎么起还在调，等稳定了再往下沉。

## 六、不查什么

文字措辞归 `language`，格式归 `docs`；装没装、新旧对不对归 `skill`。这里只管 Agent 能不能靠名字和说明找对门。
