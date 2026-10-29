# `qtfounder asset` 用户指南

资产契约、清单、关系与维护，四个入口。数据源是归位契约：十八类长期记忆，加 context 与 archive，共二十种资产。这里不另存一份资产表，改资产改 `src/meta/default-contract.toml`。

## 一、catalog：仓库里到底有没有

进一个不熟的 checkout，或新加、移除子模块之后，先看清单。

```bash
qtfounder asset catalog
```

主仓根跑，当前是 20/20：

```text
根：.
20 种资产，仓库里有 20 种，缺 0 种

context  Context 入口  data/context/  source  有
report  Report 报告  data/report/  memory  有
...
archive  Archive 归档  data/archive/  exit  有
```

从子目录跑时，`根` 会指向当前目录，结果可能是 0/20。这不是资产丢了，是根不对：

```text
根：.
20 种资产，仓库里有 0 种，缺 20 种
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

## 三、relation：资产与迁入边

```bash
qtfounder asset relation
qtfounder asset relation insight
```

不带参数按三种角色列出资产，再列关系：

```text
入口（1）
  context  data/context/
长期记忆（18）
  report  data/report/  report
  library  data/library/  library
  ...
归档（1）
  archive  data/archive/

关系
  intake  context -> memory  flip
  archive  context -> archive  same
  archive  memory -> archive  flip
```

带资产名看它的位置、类别与相关关系：

```text
insight  Insight 洞察  长期记忆
  路径：data/insight/
  类别：insight
  家族：陈述型
  同目录：report、library、history、journal、profile、brochure、roadmap、intention
  关系：
    intake  context -> memory  flip
    archive  memory -> archive  flip
```

用途：

- 判断内容该用哪条迁入边。
- 查一个资产在长期记忆里的位置与同类资产。
- 换目录或拆仓库前，先看会牵动哪些同目录资产。

只读契约，不推断资产之间的引用或依赖。

## 四、maintain：整理 context 并搬进长期记忆仓

context 与 archive 是「领域 / 资产」两级；长期记忆是资产一级仓库、领域一级文件夹。搬动时交换前两段：

```text
context/<领域>/<资产>/<余下> → <资产仓>/<领域>/<余下>
```

context 里资产在前、领域在后，也会被认出来并整理成领域在前。

```bash
qtfounder asset maintain --dry-run   # 预览要移动的文件
qtfounder asset maintain --check     # 只整理与校验，不移动；有待移动时退出码 1
qtfounder asset maintain             # 移动
qtfounder asset maintain --json      # 给脚本读
```

输出分四段：

```text
根：.
待移动 2
  life/insight/a.md -> data/insight/life/a.md
  work/journal/2026-10-27.md -> data/journal/work/2026-10-27.md
目标已存在 0
无法归位 0
目标布局异常 0
```

- `--root` 指仓根，缺省当前目录。
- 文件名等于今天（UTC）时留在 context 不移动，输出单列「当天不移动」——当天还要继续录入，隔天再搬。
- 目标已存在时不覆盖，报「目标已存在」。
- 目标仓的顶层出现资产名，说明还没做二次整理，报「目标布局异常」。
- 命令只移动文件，不提交、不推、不改子模块指针。

## 五、契约在哪

`src/meta/default-contract.toml`，查找顺序与 `meta` 一致；`--rules` 可换一份。
