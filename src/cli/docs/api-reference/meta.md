# `qtfounder meta` 接口

## 一、命令面

```text
qtfounder meta classify <PATH> [--rules PATH] [--json]
qtfounder meta classify --stdin [--rules PATH] [--json]
```

文件路径与 `--stdin` 二选一，两个都给、一个都不给都报用法错。`--rules` 换一份契约，`--json` 给机器看。

## 二、怎么判

判断层只读契约，不内置任何一格的线索。四步依次走，前一步定不下就走不到后一步。

1. 材料检查：空行分隔的段数超过 `max_paragraphs`，直接判「拆成一条再判」，不看格也不看域。
2. 定格：每格按 `cues` 计分，同一个词最多计两次。最高分低于 `min_hits` 判不收；与第二名分差小于 `min_margin` 判待定，并给出那一对格的分辨句。
3. 准入：命中的格若有一条 `transition` 同时满足 `require_any` 与不满足 `lack_any`，换成目标格，理由取该条的 `reason`。
4. 定档：按格的 `grade` 顺序找第一条命中的；都不命中就用 `fallback = true` 那条；没有分档的格（如愿景、日志）给「不分档」。

域独立判断，不影响格。领域按 `cues` 计分，同分不指路；命中数低于 `min_domain_hits` 也不指路——一个顺带出现的领域词不该把整篇搬走。命中的领域同时看 `archive_cues`：出现按归档，否则按移交。

## 三、输出

人读是一张卡，四行加一个结论：

```text
落位卡

去处：留在这里
范畴：Roadmap 路线图  data/roadmap/
档：已决策
凭什么：回答「下一步干什么」，讲先后与取舍。只陈述认识、不排先后的，不是路线图。

写法：
# 主题名
```

这一行取格的 `boundary`；发生降格时取该条 `transition` 的 `reason`，此时格也是降格后的格。

`--json` 给这些字段：

- `verdict`：`place`／`move`／`reject`／`undecided`
- `home`：去的仓，含 `name`、`label`、`form`（`移交`／`归档`）与 `reason`；留在这里时没有这个字段
- `category`：`name`、`label`、`path`、`family`、`skeleton`；不收或待定时没有这个字段
- `grade`：档；没有分档的格不出现
- `rule`：凭什么归这里，同人读那张卡
- `question`：待定时那一个问题
- `reason`：不收、待定或材料检查的原因
- `candidates`：命中过的格与分数，从高到低

## 四、退出码

- `0`：有落位，留在这里或搬走
- `1`：不收
- `2`：用法错、文件或契约读不了
- `3`：待定，等那一句的回答

待定不与失败混用，脚本可以据此停下来问人，而不是重跑。

## 五、错误

- 输入给了两个或没给——`二选一：给文件路径，或用 --stdin`
- 文件不存在——`文件不存在：notes.md`
- 契约不存在——`--rules 指的文件不存在：./my-contract.toml`
- 契约格式不对——`契约 <路径> 格式不对：…`
- 契约没有格——`契约 <路径> 没有一格：十八格至少得写进来`
- 降格规则或分辨句指向不存在的格——`契约 <路径> 的降格规则指向不存在的格：speck`
- 分辨句不成对——`契约 <路径> 的分辨句要成对给两格`

## 六、契约

你要换一份契约，按这个顺序找。先看 `--rules`；没有就看环境变量 `QTFOUNDER_META_CONTRACT`。再没有，从当前目录逐级往上找源码树里的 `src/meta/default-contract.toml`；最后看 `~/.config/qtfounder/meta/default-contract.toml`。四道都找不到就报错，不内置兜底。

字段：

| 段 | 字段 | 用途 |
|----|------|------|
| `[verdict]` | `min_margin` | 分差低于它判待定；打平就判不了 |
| | `min_hits` | 最高分低于它判不收 |
| | `min_home_hits` | 去处命中数低于它不指路 |
| | `max_paragraphs` | 超过这个段数判「拆成一条再判」 |
| | `stay` | 留在这里的说法 |
| | `archive_cues` | 出现即按归档处理 |
| | `question_fallback` | 没有现成分辨句时问那一句 |
| `domain` | 字符串 | 这份契约属于哪个域 |
| `[[home]]` | `name`／`label`／`cues`／`status` | 去处仓名、显示名、线索与确认状态 |
| `[[asset]]` | `name`／`label`／`role`／`path`／`layout` | 资产身份、角色（`source`／`memory`／`exit`）、目录与两级顺序 |
| | `description` | 两端各收什么；十八类的内容写在 `[[category]]` |
| `[[category]]` | `name`／`label`／`path`／`family` | 范畴的身份与目录 |
| | `cues`／`description`／`skeleton` | 线索、界线、写法 |
| `[[category.grade]]` | `label`／`cues`／`fallback` | 档的顺序即展示顺序；`fallback` 兜底 |
| `[[transition]]` | `from`／`to`／`require_any`／`lack_any`／`reason` | 准入不足时降格 |
| `[[question]]` | `pair`／`ask` | 两格打平时问哪一句 |
| `[[axiom]]` | `name`／`description`／`why`／`status` | 不再论证的前提；`status` 只认 `confirmed`／`suspected`，缺省怀疑 |
| | `domain` | 只认 `meta`／`asset`，缺省 `meta`；本体论层走 `meta axiom`，资产层走 `asset axiom`，写别的在读契约时拦下 |

`pair` 必须正好两格，`from` 与 `to` 必须是真格，读契约时就校验，不等到判的时候才暴露。
