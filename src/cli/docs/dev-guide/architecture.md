# 架构：验收标准与解析分离

两个子命令共用一套骨架。骨架只有一句：**验收标准只写一遍，加一门语言只写提取器。**

## 一、五段流水

```
源文件 ──提取器──> IR（事实） ──判定层──> Finding ──输出层──> 文本 / JSON
                     ▲
                 契约（阈值与词表）
```

**提取器**只回答「这个语言的语法里，函数叫什么、有几个参数、嵌了几层」。它不做判断，不读阈值。

**IR** 是与语言无关的事实表。`code` 侧是 `Facts`／`FunctionFacts`／`ClassFacts`／`BindingFacts`；`language` 侧没有 IR，因为 Markdown 本身就是事实（行、段、表、代码块）。

**判定层**只读 IR 与契约。它一行都不碰语法树——这是这条路能成立的关键，也是「加语言不改判定」的前提。

**契约**是磁盘上的 TOML。阈值、词表、落点映射、必现词全在里面，运行时可换。

**输出层**把 Finding 变成人能读的分组摘要，或机器能读的 JSON。

## 二、为什么 code 要 IR，language 不要

判断要不要引入 IR，只看一条：**同一个事实会不会被两种以上语法表达**。

函数有几个参数，Rust 写在 `fn` 签名里，Go 写在 `parameter_list` 里，Dart 还要先钻进 `signature`——不统一就判不了。所以 `code` 必须有 IR。

一段文字是什么手气，Markdown 与纯文本没有语法分歧，直接判即可。硬加一层 IR 只会让「加一层」变成「加三处」。

## 三、Rust 用 syn，别的语言用 tree-sitter

Rust 自己的语法走 `syn`：给的是带类型 AST，判起来比 tree-sitter 的通用节点名可靠。

别的语言没有这个待遇，于是统一走 tree-sitter，差异全收进 `src/code/extract/specs.rs` 的 `LangSpec`：节点名、下钻路径、参数条目节点、嵌套节点、错误信息候选节点，一共十七个字段。加一门语言就是加一份结构体字面量。

## 四、两个子命令的对称性

| | `code` | `language` |
|---|---|---|
| 介质 | 源码 | Markdown |
| 层数 | 4 | 7 |
| 契约 | `src/code/default-contract.toml` | `src/language/default-contract.toml` |
| 环境变量 | `QTFOUNDER_CODE_CONTRACT` | `QTFOUNDER_LANGUAGE_CONTRACT` |
| 语义层 | 无 | `--sense` |

不对称的地方都写明了理由，没有一处是「先这样吧」。

## 五、契约不许编译进二进制

第一版把契约用 `include_str!` 打进二进制，代价是改了阈值要重编译。现在改成运行时按三个来源找文件：`--rules`、环境变量、从当前目录往上找。

找不到就以退出码 2 明确报错，**不内置兜底**。悄悄回退到默认值，等于让用户以为阈值生效了。
