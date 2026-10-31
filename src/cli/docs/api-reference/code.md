# `qtfounder code` 接口

## 一、命令面

```
qtfounder code reflect [NAME] [--example] [--json] [--rules PATH]
qtfounder code review [ROOT] [--example] [--json] [--rules PATH] [--record --user NAME] [--experiences]
```

`code` 必须给一个子命令（`reflect` 或 `review`），不再有缺省的语用检查——那四层（`lexical`／`syntax`／`structure`／`pragmatic`）已并入 `qtfounder language` 的编程语言侧。

### `reflect`：任务完成后的架构反思清单

**NAME**：只看某一条（按 `name`），缺省四条全打。名字打错以退出码 2 报错，并列出认识的 name。

**`--example`**：连 `example` 层一起打。缺省只给 description 与 why——要不要例子按需，缺省省掉。

**`--json`**：每条输出 `{name, description, why, example}` 四层，缺省全给（JSON 不缺例）。

`reflect` 不扫描代码、不产生命中，正常退出恒为 `0`；契约读不到、没有 `[[reflect]]`、name 不认识，以退出码 2 报错。

### `review`：功能审查骨架

**ROOT**：要审的目录或文件，缺省当前目录。目标不在以退出码 3 报错。目录会递归找 `.rs`／`.py`／`.go`／`.ts`／`.mts`／`.cts`／`.dart`，跳过契约 `[scope]` exclude 里的目录。

**`--example`**：连六条纪律的 `example` 一起打。

输出四节骨架（结论／功能预期／清单与锚点／建议）与六条交付纪律；清单里是事实面取来的功能锚点，每条带 `file:line`，**不排序、不标严重度**——功能对不对对着预期判，归写报告的人。

功能锚点四种：`function`（函数清单，带参数与行数）、`dead_branch`（恒假条件）、`complex`／`interface`（复杂度过高、参数过多）、`thin_error`（错误信息太短）。复杂度与错误信息阈值取契约 `[syntax]` 与 `[pragmatic]`。锚点计数进退出码：有命中退 `1`，零命中退 `0`。

**`--record`**：记一条使用体会，正文从标准输入读，配合 `--user` 指定是谁的体会；缺 `--user` 以退出码 2 报错。

**`--experiences`**：列出这个目标已记的使用体会（多个用户），不出骨架。

JSON 输出 `{root, functional_questions, rules, anchors, experiences}`；`functional_questions` 是功能审查的四问（解决什么／承诺什么／边界覆盖没／哪没验证）。

## 二、退出码

退出码 `0` 表示无命中，`1` 表示有命中，`2` 表示用不了——契约读不到、记体会缺 `--user` 或有文件解析失败，`3` 表示目标缺席（路径不存在）。

**解析失败不影响其他文件**：坏文件在标准错误输出里点名，已成功的部分照常报，退出码最后置 2。

## 三、契约

默认契约是磁盘上的文件 `src/cli/src/code/default-contract.toml`，查找顺序：`--rules` → 环境变量 `QTFOUNDER_CODE_CONTRACT` → 从当前目录往上找 `src/code/default-contract.toml` 或 `src/cli/src/code/default-contract.toml` → 用户级 `~/.config/qtfounder/code/default-contract.toml`。刻意不内置兜底：找不到以退出码 2 报错。

`review` 用其中的 `[[review]]`（六条交付纪律）与 `[syntax]`／`[pragmatic]`（复杂度与错误信息阈值）；`reflect` 用 `[[reflect]]`。编程语言四层的语言学阈值（`[lexical]`／`[syntax]`／`[module]`／`[pragmatic]`）也在这份契约里，由 `language` 的编程语言侧读取。

## 四、体会存储

`--record`／`--experiences` 读写 `<数据根>/review/experiences.json`，与 `write review` 的读者体会同一文件，靠 `domain`（`code`／`write`）区分。数据根解析：`QTFOUNDER_DATA` → `$XDG_DATA_HOME/qtfounder` → `~/.local/share/qtfounder`。
