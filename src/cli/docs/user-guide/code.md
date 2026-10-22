# `qtfounder code` 用户指南

代码语用检查：判的不是语法错、不是风格、不是圈复杂度，而是**下一个维护者能不能看懂**。

## 一、四层查什么

| 层 | 问的问题 | 判据举例 |
|---|---|---|
| 词汇层 | 名字有信息量吗、名实相符吗 | 万能名 `data`／`temp`／`flag`；缩写 `ctx` 有两种读法；`user_map` 里装的是 `Vec`；仓库里 `get`／`fetch`／`load`／`read` 并存 |
| 句法层 | 读者要同时记住多少东西 | 嵌套超 3 层；函数超 60 行；参数超 5 个；认知负荷超 15；一行链 5 层调用 |
| 语篇层 | 一个文件在讲几件事 | 顶层定义超 15 个；一个 impl 方法超 15 个；`if false` 死分支 |
| 语用层 | 意图传达到了吗 | `handle` 不说对象；文档注释只把函数名说一遍；复杂分支零注释；`panic!("bad")` 不说怎么修 |

每条命中都写清违反 Grice 的哪一条原则（量／质／关系／方式）——这是它跟风格检查器的分别。

## 二、用法

```bash
cargo run -- code                 # 查当前目录
cargo run -- code src/cli         # 查指定目录
cargo run -- code src --layer pragmatic
cargo run -- code src --top 2     # 每类最多两条
cargo run -- code src --json
cargo run -- code src --rules my-rules.toml
```

装出来的二进制是 `qtfounder`，命令即 `qtfounder code ...`。

退出码：**0 无命中 / 1 有命中 / 2 有文件解析失败**。

## 三、阈值在哪改

内嵌默认表在 `src/cli/data/rules/code.toml`（编译进二进制），`--rules` 可覆盖。

- `[lexical]` —— 万能名词表、短名长度、白名单、缩写表、同义组；
- `[syntax]` —— 五个阈值：嵌套、行数、参数、认知负荷、链长；
- `[module]` —— 顶层定义数、impl 方法数；
- `[pragmatic]` —— 错误信息最短长度、复杂函数多少行才要求注释。

**名字好坏是项目口径**，所以词表在规则表里，不写死在代码里。

## 四、和 rustc 的分工：哪些判据**不做**

`unused_imports` 不判——rustc 的这项 lint 已经权威、零误报；靠 AST 自己判会漏掉 derive 宏与 trait 方法这两种用法（实测第一版把 `#[derive(Parser)]` 与 `use syn::spanned::Spanned` 全报成未用）。**重复做一遍只会更差。**

同理不做：语法错（rustc）、格式（rustfmt）、安全与性能（clippy 与其他工具）。

## 五、边界

不判逻辑对不对（测试的事）、不判「这段该不该存在」、不判类型标注是否最优。名字与结构判据都在**读者要不要记住它**这个前提下成立，不是审美偏好。

## 六、已知的判据近似

- 「短名」按作用域判：循环变量、闭包参数、match 绑定豁免——它们只活几行；
- 「认知负荷」按嵌套深度加权累加，是 Sonar 那套的近似，不是圈复杂度；
- 「名实不符」只看 `let` 初始化的构造器名字（`vec!`／`HashMap::new`／`String::from`），类型推断出来的情况看不到；
- 「文档复述」按词重合判，可能在极简文档上误报，所以只在文档短于 40 字时判。
