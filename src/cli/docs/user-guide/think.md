# `qtfounder think` 用户指南

认知这一侧的命令。`code`／`language`／`docs` 查的是产出物，`think` 查的是怎么想、怎么应对。

三个入口：`load` 从会话语料里看消耗落在哪，`reflect` 按模型记一条反思并到点回收，`model` 把当前生效的模型摊开看。

## 一、`load`：这股消耗落在哪

读 Hermes 会话库里的用户消息，筛掉过短与纯确认，分批送模型判断，最后按环节汇总。判的不是「情绪好不好」，而是消耗来自哪里、落在助手的哪个环节。

信号只有两类：`fatigue` 是自己的精力被耗掉，`friction` 是事情反复、被替做决定、信息对不上带来的烦躁。环节固定五个，全部记在助手一侧：

- `A`：没照做、忘了、擅自改
- `B`：输出太复杂、看不懂
- `C`：没给判据、没法验证
- `D`：没把选项收敛到能拍
- `E`：没把外部条件的坑先探掉

每条命中都要挂一句原话，截取六到四十字，不许改写；没有原话就不算命中。命中数超过 `--max-hits` 就以退出码 1 结束，所以能定期跑。

```bash
qtfounder think load --since 2026-10-01
qtfounder think load --dry-run          # 只统计候选量，不调模型
```

需要会话库和模型凭据。口径在 `prompt.md` 里，不在代码里。

## 二、`reflect`：记一条可检验的判断

认知行为疗法把一次卡住的念头拆成五栏：情境、情绪、自动思维、正反证据、替代思维，最后复评情绪强度。这套结构有效不在栏多，而在每一栏都能被事实顶住：「他会谈砸」是判断，「团队谈过三次只差一个点」是事实。

再挂一个行为实验，反思才闭环。写下一条能被推翻的预测，定一个复看日，到点回填实际发生了什么。创始人版的放手实验就是这个：「我不碰两周，会出事吗」。

两个诊断问题落在 `domain` 与 `control` 上。前者判这件事属于公司还是属于我这个人，后者判是真的只有我能做（`scarce`）还是我不放心（`grasp`）。生活侧的事只填 `domain`，硬套一个控制属性反而失真。

四个入口互斥，一次只用一个：

```bash
qtfounder think reflect --stdin --json        # 记一条
qtfounder think reflect --list                # 待复看与未填完
qtfounder think reflect --id ref_3WT3         # 看一条
qtfounder think reflect --observe ref_3WT3 --result "只错了一个报价点"
```

记完给三样东西。`missing` 是还空着的栏，`next_question` 是照第一处空缺给出的下一问，`recurring` 是出现两次以上的认知扭曲。追问由 agent 在对话里做，工具不追问。

认知扭曲只用来帮人看清念头，不用来贴标签。词表在模型里，`think model` 能看到当前那一份。

## 三、`model`：改模型不用改代码

栏位顺序、追问用语、认知扭曲词表、归属与控制的取值域，以及 `load` 宽筛用的信号词与长度门槛，全部在模型文件里。默认读仓库里的 `src/think/default-contract.toml`；装到用户级目录后读 `~/.config/qtfounder/think/model.toml`。

```bash
qtfounder think model            # 人读
qtfounder think model --json     # 给机器看
```

```text
认知模型：src/think/default-contract.toml
宽筛：46 个信号词，无信号时短于 15 字不进候选
归属：company / self
控制：scarce / grasp
栏位顺序：emotion → distortions → evidence_against → balanced → experiment
认知扭曲：全或无、灾难化、读心、应该式、个人化、情绪化推理、贬低积极面、过度概括、贴标签、心理过滤
收尾问：到复看日回填结果：实际发生的事，比任何推理都算数。
```

与其他三份契约不同，认知模型有内置兜底：`reflect` 是记录本，不该因为找不到配置文件就记不了账。

## 四、什么时候别记

默认不留痕，说完就结束。值得记的是带判断和动作的事：要不要接、要不要亲自做、放不放手。纯感受的记录留在日志里，不必变成一条实验。

日志里读得到的是事后叙述，不是当时那句话。所以自动思维必须由 agent 提出候选、本人确认才能写进去——不问就写，等于替人编念头，整条记录的根就是假的。反对证据和替代思维通常已经写在同一天稍后几行，先翻日志再问人。

## 五、落在哪

`load` 读 `~/.hermes/state.db`，不写任何东西。`reflect` 的账本在 `<数据目录>/think/reflect.json`，数据目录取 `QTFOUNDER_DATA`，其次 `$XDG_DATA_HOME/qtfounder`，再次 `~/.local/share/qtfounder`。
