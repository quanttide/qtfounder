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

这批命中就是本工具的用法示范。逐条读下来能分成三类：

- **真该改的**：`run` 这种只说动作、不说对象的函数名；`"写 samples"` 这种太短的错误信息；抽取值那段六十三行没一句说明为什么（`missing_why` 命中在 `code/extract/rust_lang.rs`）；
- **验收标准太紧的**：阈值该按项目的量法调，不是按工具的默认值；
- **验收标准错的**：改验收标准，并在 `src/code/tests.rs` 里补一条回归测试，否则下一轮又会报。

**每修一类误报就留一条测试**——这条比任何文档都管用。

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

**加语言只写配置，`判定层`一行不改**——这是这条路成立的唯一验收。三种新语言暂时缺三条验收标准：调用链长度、文档复述、名实不符。缺就写明，别当成没问题。

## 五、和 rustc 的分工

`unused_imports` 不判。rustc 这项已经权威且零误报，靠语法树自己判会漏掉 derive 宏与 trait 方法这两种用法（第一版实测把 `#[derive(Parser)]` 全报成未用）。重复做一遍只会更差。

同理不做：语法错、格式、安全与性能。

## 六、契约在哪

默认契约是磁盘上的文件：`src/cli/src/code/default-contract.toml`。查找顺序是 `--rules`、环境变量 `QTFOUNDER_CODE_CONTRACT`、从当前目录往上找。

刻意不内置兜底：找不到就以退出码 2 报错。契约是显性文件，它的数值就是唯一的量法。
