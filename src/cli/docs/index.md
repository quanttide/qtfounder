# qtfounder CLI 文档

命令行工具，八个子命令：`code` 查源码，`language` 查文字，`docs` 查文档结构，`think` 管认知，`meta` 判归位，`skill` 装 Skill，`agent` 查 Skill 写法，`audit` 出体检报告。

## 用户指南

- [`code`](user-guide/code.md)：代码语用检查，四层。先看它怎么查它自己（八十五处命中分四层）。
- [`language`](user-guide/language.md)：语言检查，六层加语义层，出结果再过一层复核。
- [`docs`](user-guide/docs.md)：文档审计，表格、列表、密度、加粗与分割线五层；阈值未标定前密度只观测。
- [`think`](user-guide/think.md)：认知侧，`load` 看消耗落点、`reflect` 记一条反思、`model` 看模型。
- [`meta`](user-guide/meta.md)：归位，`classify` 给一段内容一张落位卡：域、格、档、写法。
- [`skill`](user-guide/skill.md)：把二进制里嵌的 Skill 装到目标仓，`list` 看状态、`check` 查差异。
- [`agent`](user-guide/agent.md)：查 Skill 的名字、说明与正文：范围-动词、触发时机、一级标题。
- [`audit`](user-guide/audit.md)：文字体检，合并两套审计，每期只说一处。

## 开发指南

- [架构](dev-guide/architecture.md)：五段流水，为什么 `code` 要 IR 而 `language` 不要。
- [加一门语言](dev-guide/code/adding-a-language.md)：只写一份配置，`判断层`一行不改。
- [加一条验收标准](dev-guide/adding-a-check.md)：一条检查住在四处，另加三种不该加的验收标准。
- [测试](dev-guide/testing.md)：真样本、反样本、契约可变，以及并行测试的临时文件坑。
- [路线图](dev-guide/roadmap.md)。

## 接口参考

- [`code` 接口](api-reference/code.md)：参数、退出码、JSON 字段、稳定性承诺。
- [`language` 接口](api-reference/language.md)：同上，加语义层的环境变量与降级办法。
- [`docs` 接口](api-reference/docs.md)：三层参数、契约字段与 `docs check` 的退出码。
- [`think` 接口](api-reference/think.md)：三个子命令的参数、输入字段、JSON 与错误表。
- [`meta` 接口](api-reference/meta.md)：怎么判、落位卡字段、退出码与契约结构。
- [`skill` 接口](api-reference/skill.md)：嵌入清单、装到哪、JSON 与退出码。
- [`agent` 接口](api-reference/agent.md)：七道检查、级别与行号、契约字段。
- [`audit` 接口](api-reference/audit.md)：参数、退出码与报告 JSON。

## 一条自举约定

改了工具就回头跑一遍自己的文档：

```bash
cargo run -- language docs
cargo run -- docs docs
cargo run -- code src
```

三条都还没清零：`language docs` 审十六个文件、报四十处（真问题二十八处，另十二处被复核层标成误报），`docs docs` 已清零，`code src` 五十二处。

代码侧剩的欠账，你自己跑一遍 `code src` 就能看到：文件里顶层定义太多，`check_table` 与 `internal_ids` 两个函数过长，命令行入口又长又深。这几条是结构问题，不是措辞问题，要拆就另开一轮。文档侧的表格嫌疑集中在接口参考的字段表，那是真在对照，不改。

## 已知边界

下面四条是 2026-10-22 查 `categories/quanttide-handbook` 时确认的，你动这块代码前先看它们。

审哪些文件由你给的 ROOT 决定，工具不再内嵌目录白名单。2026-10-22 之前 `include_default` 里写着五个目录名，那等于把仓的章程抄进工具：`categories/quanttide-handbook` 的正文在 `domains/`、`default/` 下，五条全落空，命令审 0 个文件、打一句「没有命中」，看着像通过。现在默认审 ROOT 下全部 `.md`，只跳过 `[scope] exclude`（流水、报告、归档、示例、第三方依赖、变更日志），`--scope all` 连 exclude 也不放过。exclude 里带 `/` 的按路径前缀算，不带的按同名文件或目录算，所以 `CHANGELOG.md` 写一条就覆盖嵌套的那几份。代码层 `code` 跳过同一批目录（`archive`、`examples`），两个子命令的算法一致。输出的第一行会报「审了 N 个文件」，那个数字是 0 就说明范围没设对。

提示语里的比例是写死的字面量。`scan.rs:202` 打出的「他在 60 字以上只占 7%」，运行时并不去算 `data/profile` 的分布，你的语料变了它照样这么印。

句子切分不认加粗小标题。`progressive-delegation.md` 第 10 行报「的」四个，其中一个是加粗小标题「执行自己的想法」里的。复核层认这一形态并标「误报·标题并入」，但切分本身没改，句子成分里仍然带着标题。

禁用词分不出「拿它写」和「讲它」。复核层能把引号里、反引号里出现的词判成「提及」。同一个词当行内术语直接用时它判不出来：`resignation.md` 里「各项职责和项目的「接手人」对交接文档进行评审」说的是交接角色本身，去掉引号就成了 ERROR。

工具判不了体裁、词的多义、某句是引述还是自述。第一项在 `pitfalls.md` 上最明显，该文件八十一处命中里声量最大的 voice 七条，恰好是实证流水最该原样保留的地方。契约里已有结论：机判不了，人判一次。

已经做到的是四样。每条禁用词带日期与出处原话，所以一处词禁是存量还是新写、该不该动，都查得清，不用猜工具的口味。单个文件作参数时必审，不受范围过滤。`--kind`、`--layer`、`--top` 能把一批命中按层拆开，你看到的是分层账，不是一坨数。复核层把「提及、引用输出、标题并入」三种规则误报单独标出来并计数，误报不隐藏、也不算通过。
