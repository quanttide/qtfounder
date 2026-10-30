# `qtfounder code` 用户指南

代码语用检查：判的不是语法错、不是风格、不是圈复杂度，而是**下一个维护者能不能看懂**。

## 一、先用它查它自己

文档最省事的验证法是自举——拿它查它自己的源码：

```bash
cargo run -- code src
```

跑出来的摘要长这样：

```
合计 64 处（词汇层 9，句法层 27，语篇层 7，语用层 21）
```

这批命中就是本工具的用法示范，逐条读下来能分成三类。

第一类是真该改的：`run` 这种只说动作、不说对象的函数名，`"写 samples"` 这种太短的错误信息，以及抽取值那段六十三行没一句说明为什么（`missing_why` 命中在 `code/extract/rust_lang.rs`）。第二类是验收标准太紧，阈值该按项目的量法调，不是按工具的默认值。第三类是验收标准本身写错了，那就改标准，并在 `src/code/tests.rs` 里补一条回归测试，否则下一轮又会报。

每修一类误报就留一条测试，这条比任何文档都管用。

## 二、四层查什么

**词汇层**：`data`／`temp`／`flag` 这类万能名；`ctx` 有两种读法的缩写；`user_map` 里装的是 `Vec`；仓库里 `get`／`fetch`／`load`／`read` 并存。

**句法层**：嵌套超三层、函数超六十行、参数超五个、认知负荷超十五、一行链五层调用。

**语篇层**：顶层定义超十五个、一个 impl 方法超十五个、`if false` 死分支。

**语用层**：`handle` 不说对象；文档注释只把函数名说一遍；复杂分支零注释；`panic!("bad")` 不说怎么修。

每条命中都写清违反 Grice 的哪一条原则，这是它跟风格检查器的分别。

## 三、用法

```bash
cargo run -- code                       # 查当前目录
cargo run -- code src --layer pragmatic # 只看一层
cargo run -- code src --top 2           # 每类最多两条
cargo run -- code src --json            # 给机器读
cargo run -- code src --rules my.toml   # 换契约
```

退出码：**0 无命中 / 1 有命中 / 2 有文件解析失败**。

## 四、支持哪些语言

`rs` 走 `syn`；`py` 走 tree-sitter-python；`go`、`ts`／`mts`／`cts`、`dart` 共用一份通用 tree-sitter 提取器，差异全在 `src/code/extract/specs.rs` 的 `LangSpec` 里。

**加语言只写配置，`判断层`一行不改**——这是这条路成立的唯一验收。三种新语言暂时缺三条验收标准：调用链长度、文档复述、名实不符。缺就写明，别当成没问题。

## 五、和 rustc 的分工

`unused_imports` 不判。rustc 这项已经权威且零误报，靠语法树自己判会漏掉 derive 宏与 trait 方法这两种用法（第一版实测把 `#[derive(Parser)]` 全报成未用）。重复做一遍只会更差。

同理不做：语法错、格式、安全与性能。

## 六、契约在哪

默认契约是磁盘上的文件：`src/cli/src/code/default-contract.toml`。查找顺序是 `--rules`、环境变量 `QTFOUNDER_CODE_CONTRACT`、从当前目录往上找。

刻意不内置兜底：找不到就以退出码 2 报错。契约是显性文件，它的数值就是唯一的量法。

## 七、reflect：任务完成后的架构反思

```bash
qtfounder code reflect              # 打反思清单（看什么、为什么算缺陷）
qtfounder code reflect --example    # 连例子一起打，例子按需
qtfounder code reflect boundary-creep   # 只看一条
qtfounder code reflect --json       # 给脚本读
```

**它只给清单，不下结论。** 对着刚改的代码哪几条成立是判断，归 AI；清单是四道问答题，答「不是」就翻篇，不硬凑。四条全空也是结论，照实写。

四条按顺序排，顺序即优先级：

1. `unmarked-additions` —— 这次改动里有没有哪一行不是你要求的，也没写进意图的哪一层。
2. `unresolved-piling` —— 意图声明的不确定项，这次加了几条、解了几条。
3. `boundary-creep` —— 这次改动有没有落在意图声明写「不干」的范围里。
4. `patch-over-framework` —— 这次是把现有框架加固了，还是在框架外面又补了一块。

**发现只进路线图** `data/roadmap/qtfounder/index.md`（独立 git 仓），不在这儿存，也不在 `data/` 下别的文件存第二份。整套走法见 Skill `code-reflect`：拿清单、逐条回看、按 Roadmap 维护手册的三个自检收口、提交并复跑。

反思清单是契约的一部分，加规则改 `[[reflect]]` 段即可，改了要同步 skill 正文与 `src/code/tests.rs` 的条数——两处钉着同一批数，漂移报得出来。

## 八、review：意图审查

```bash
qtfounder code review src/meta           # 报告骨架 + 语用命中的 file:line 锚点
qtfounder code review src/meta --layer pragmatic   # 只取某几层的命中
qtfounder code review src/meta --example # 连纪律的例子一起打
qtfounder code review src/meta --json    # 给脚本读
```

审查是意图驱动编程的**反向流程**：从代码反推写它时的意图，再看还对不对得上。命令只出两样——

- **四节骨架**：结论、意图假设（核心目标／边界／约束／已知假设／不确定性）、偏移清单、建议；
- **锚点**：语用检查的命中自带 `file:line`，正是报告要求的锚点格式。

**锚点不排序、不标严重度**。严重度得对着意图判，那是判断，归写报告的人；意图假设那五节命令一个字不填——反推也是判断。这条分工与 `reflect` 一致：机械层能给的只有锚点。

交付纪律六条写在契约 `[[review]]`：结论先行、锚点必备（写不出就不写）、一条一个发现、按严重度排、只给候选不给结论、不写过程。与反思的分工：反思看「这轮干完多了什么」，审查看「这段代码还对不对得上意图」。整套走法见 Skill `code-review`。
