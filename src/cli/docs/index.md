# qtfounder CLI 文档

命令行工具，两个子命令查两类介质：`code` 查源码，`language` 查文字。

## 用户指南

- [`code`](user-guide/code.md)：代码语用检查，四层。先看它怎么查它自己（六十四处命中分三类）。
- [`language`](user-guide/language.md)：语言检查，七层加语义层。先看它怎么查本仓文档（零命中）与一个真语料仓（一千二百九十九处）。

## 开发指南

- [架构](dev-guide/architecture.md)：五段流水，为什么 `code` 要 IR 而 `language` 不要。
- [加一门语言](dev-guide/adding-a-language.md)：只写一份配置，判定层一行不改。
- [加一条验收标准](dev-guide/adding-a-check.md)：一条检查住在四处，另加三种不该加的验收标准。
- [测试](dev-guide/testing.md)：真样本、反样本、契约可变，以及并行测试的临时文件坑。
- [路线图](dev-guide/roadmap.md)。

## 接口参考

- [`code` 接口](api-reference/code.md)：参数、退出码、JSON 字段、稳定性承诺。
- [`language` 接口](api-reference/language.md)：同上，加语义层的环境变量与降级口径。

## 一条自举约定

改了工具就回头跑一遍自己的文档：

```bash
cargo run -- language docs
cargo run -- code src
```

两条都还没清零：`language docs` 现在审十二个文件、报二十四处（其中二十处是 voice），`code src` 六十四处。

## 已知边界

下面四条是 2026-10-22 查 `categories/quanttide-handbook` 时确认的，你动这块代码前先看它们。

审哪些文件由你给的 ROOT 决定，工具不再内嵌目录白名单。2026-10-22 之前 `include_default` 里写着五个目录名，那等于把仓的章程抄进工具：`categories/quanttide-handbook` 的正文在 `domains/`、`default/` 下，五条全落空，命令审 0 个文件、打一句「没有命中」，看着像通过。现在默认审 ROOT 下全部 `.md`，只跳过 `[scope] exclude`（流水、报告、归档、示例、第三方依赖、变更日志），`--scope all` 连 exclude 也不放过。exclude 条目带 `/` 的按路径前缀算，不带的按同名文件或目录算，所以 `CHANGELOG.md` 写一条就覆盖嵌套的那几份。代码层 `code` 跳过同一批目录（`archive`、`examples`），两个子命令口径一致。输出的第一行会报「审了 N 个文件」，那个数字是 0 就说明范围没设对。

提示语里的比例是写死的字面量。`scan.rs:202` 打出的「他在 60 字以上只占 7%」，运行时并不去算 `data/profile` 的分布，你的语料变了它照样这么印。

句子切分不认加粗小标题。`progressive-delegation.md` 第 10 行报「的」四个，其中一个是加粗小标题「执行自己的想法」里的。

禁用词只认词形，不认语境。`[banned."接手人"]` 的出处是「写作即交接」里自造，但 `resignation.md` 里「各项职责和项目的接手人对交接文档进行评审」说的是交接角色本身，仍被打成 ERROR。

工具判不了体裁、词的多义、某句是引述还是自述。第一项在 `pitfalls.md` 上最明显，该文件八十一处命中里声量最大的 voice 七条，恰好是实证流水最该原样保留的地方。契约里已有结论：机判不了，人判一次。

已经做到的是三样。每条禁用词带日期与出处原话，所以一处词禁是存量还是新写、该不该动，都查得清，不用猜工具的口味。单个文件作参数时必审，不受范围过滤。`--kind`、`--layer`、`--top` 能把一批命中按层拆开，你看到的是分层账，不是一坨数。
