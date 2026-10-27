# `qtfounder asset` 用户指南

资产契约、清单与关系，三个都是只读视图。数据源是归位契约：十八格就是十八类资产。这里不另存一份资产表，改资产改 `src/meta/default-contract.toml`。

## 一、catalog：仓库里到底有没有

进一个不熟的 checkout，或新加、移除子模块之后，先看清单。

```bash
qtfounder asset catalog
```

主仓根跑，当前是 18/18：

```text
根：.
18 类资产，仓库里有 18 类，缺 0 类

report  Report 报告  data/report/  陈述型  有
library  Library 图书馆  data/library/  陈述型  有
...
laboratory  Laboratory 实验室  examples/  程序型  有
```

从子目录跑时，`根` 会指向当前目录，结果可能是 0/18。这不是资产丢了，是根不对：

```text
根：.
18 类资产，仓库里有 0 类，缺 18 类
```

用 `--root` 指回仓根：

```bash
qtfounder asset catalog --root ../..
qtfounder asset catalog --present      # 只列已有的
qtfounder asset catalog --json         # 给脚本读
```

只查路径在不在，不查目录里有没有内容，也不判断内容合不合契约。

## 二、contract：写之前先看边界

```bash
qtfounder asset contract insight
```

```text
insight  Insight 洞察  data/insight/  陈述型
  边界：一句话能被推翻，推翻之后做法会变；抽掉日期和当事人还成立。
  分档：已确认（已经、做了、落地、改成、多次、重复、第二次）；假说（默认）
  写法：
    # 主题名

    不规定章节。开篇说清这条要回答什么、推翻什么；正文按论证需要组织；可信度与推翻条件写进论证；结尾落一句能据以行动的判断。
```

用来回答三类问题：

- 写新资产之前，这类内容的边界在哪。
- 评审一条内容时，它是不是放错了类。
- 同一条判断落进来之后，算「已确认」还是「假说」。

不带参数会列出十八类资产，155 行左右；查一类把资产名放在后面即可。`--json` 给工具读。

这是契约的展开，不是写作模板。陈述型不规定章节，模板只归 `docs/` 下的手册与规格。

## 三、relation：和谁一类、和谁同目录

```bash
qtfounder asset relation
qtfounder asset relation insight
```

不带参数按两条线分组：

```text
陈述型（9）
  report  data/report/
  ...
程序型（9）
  bylaw  docs/bylaw/
  ...
按目录
  data/  report、library、history、journal、profile、brochure、roadmap、insight、intention
  docs/  bylaw、specification、handbook、gallery、tutorial、essay
  packages/  toolkit
  apps/  platform
  examples/  laboratory
```

带资产名看它和谁住在一起：

```text
insight  Insight 洞察
  家族：陈述型
  目录：data/
  同目录：report、library、history、journal、profile、brochure、roadmap、intention
```

用途：

- 判断一类新资产该归哪个家族、哪个目录。最终仍以契约里的 `path` 为准。
- 比较同类资产：同目录的资产通常共享发布时间线与维护方式。
- 换目录或拆仓库前，先看会牵动哪些同目录资产。

只读契约的 `family` 与路径首段，不推断资产之间的引用或依赖。

## 四、契约在哪

`src/meta/default-contract.toml`，查找顺序与 `meta` 一致；`--rules` 可换一份。
