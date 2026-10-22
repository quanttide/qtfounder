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

第一条要求零命中；第二条目前有六十四处，是待改清单。
