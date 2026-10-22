# 开发指南

通用部分在根下，模块特定的进各自文件夹。

## 通用

- [架构](architecture.md)：五段流水，为什么 `code` 要 IR 而 `language` 不要。
- [加一条验收标准](adding-a-check.md)：一条检查住在四处，另加三种不该加的判据。
- [测试](testing.md)：真样本、反样本、契约可变，以及并行测试的临时文件坑。
- [路线图](roadmap.md)。

## `code` 模块

- [加一门语言](code/adding-a-language.md)：只写一份配置，判定层一行不改。

## `language` 模块

- [语义层怎么开发](language/sense-layer.md)：提示词在磁盘上、`temperature=0` 的理由、为什么不用外部 SDK。

判断一份文档放哪：**只对某一个子命令成立的，进该模块文件夹；两个子命令都适用的，留根下。** 拿不准就看例子里出现的是哪种介质。
