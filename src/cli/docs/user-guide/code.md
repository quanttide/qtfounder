# `qtfounder code` 用户指南

`code` 有两道：`reflect` 任务后反思、`review` 功能审查。两道都出骨架与锚点，判断归人。

**代码的语言学特点（命名、句法、语篇、语用四层）不在这里**——那判的是「下一个维护者能不能看懂」，已并入 `qtfounder language`，它同时判自然语言与编程语言的语言学特点。本命令只管功能：代码该做什么、还对不对得上。

## 一、reflect：任务完成后的架构反思

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

**发现只进路线图** `data/roadmap/qtfounder/index.md`（独立 git 仓），不在这儿存第二份。整套走法见 Skill `code-reflect`。

## 二、review：功能审查

```bash
qtfounder code review src/meta           # 报告骨架 + 事实面的功能锚点（带 file:line）
qtfounder code review src/meta --example # 连纪律的例子一起打
qtfounder code review src/meta --json    # 给脚本读
```

审查看的是**功能还对不对得上**，不是代码写得好不好（读感、命名归 `language`）。命令只出两样——

- **四节骨架**：结论、功能预期（解决什么／承诺什么／边界覆盖没／哪没验证）、清单与锚点、建议；
- **功能锚点**：从事实面（`Facts`）取来的四种，每条带 `file:line`——函数清单（每个函数的参数与行数）、恒假条件（永远不会执行的分支）、复杂度过高（嵌套深、认知负荷大、参数多）、错误信息太短（失败时用户不知道怎么办）。

**锚点不排序、不标严重度**。功能对不对要对着预期判，那是判断，归写报告的人；功能预期那四节命令一个字不填——推预期也是判断。这条分工与 `reflect` 一致：机械层能给的只有事实。

交付纪律六条写在契约 `[[review]]`：结论先行、锚点必备（写不出就不写）、一条一个发现、按严重度排、只给候选不给结论、不写过程。与反思的分工：反思看「这轮干完多了什么」，审查看「这段功能还对不对得上」。整套走法见 Skill `code-review`。

### 记使用体会

审完，把你自己（或不同使用者）的真实判断记下来，一人一条。同一处代码可以有多个人各记各的，下次审时摊开看。

```bash
echo "函数清单很有用，但恒假那处我没看出是不是漏了分支。" \
  | qtfounder code review <目标> --record --user alice
qtfounder code review <目标> --experiences        # 列出所有人的体会
qtfounder code review <目标> --experiences --json  # 给脚本读
```

体会与 `write review` 的读者体会落在同一处（`<数据根>/review/experiences.json`），靠 domain 区分。体会是人的话，机器不代笔、不改写。报告里有一节「使用体会」，记过的会摊在那儿。

## 三、支持哪些语言

`rs` 走 `syn`；`py` 走 tree-sitter-python；`go`、`ts`／`mts`／`cts`、`dart` 共用一份通用 tree-sitter 提取器，差异全在 `src/code/extract/specs.rs` 的 `LangSpec` 里。这些提取器同时供 `language` 的编程语言侧复用——加语言只写配置，判断层一行不改。

## 四、和 rustc 的分工

`unused_imports` 不判。rustc 这项已经权威且零误报，靠语法树自己判会漏掉 derive 宏与 trait 方法这两种用法。重复做一遍只会更差。

同理不做：语法错、格式、安全与性能。

## 五、契约在哪

默认契约是磁盘上的文件：`src/cli/src/code/default-contract.toml`。查找顺序是 `--rules`、环境变量 `QTFOUNDER_CODE_CONTRACT`、从当前目录往上找。

刻意不内置兜底：找不到就以退出码 2 报错。契约是显性文件，它的数值就是唯一的量法。功能审查用其中的 `[[review]]`（纪律）与 `[syntax]`／`[pragmatic]`（复杂度与错误信息阈值）。
