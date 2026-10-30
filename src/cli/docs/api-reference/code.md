# `qtfounder code` 接口

## 一、命令面

```
qtfounder code [ROOT] [--layer LAYER]... [--top N] [--json] [--rules PATH]
qtfounder code reflect [NAME] [--example] [--json] [--rules PATH]
qtfounder code review [ROOT] [--layer LAYER]... [--example] [--json] [--rules PATH]
```

### 缺省：语用检查

**ROOT**：目录或单文件，缺省当前目录。目录会递归找 `.rs`／`.py`／`.go`／`.ts`／`.mts`／`.cts`／`.dart`，跳过 `target`／`.git`／`node_modules`／`build`／`dist`。

**`--layer`**：`lexical`／`syntax`／`structure`／`pragmatic`，可重复。缺省四层全跑。写错层名以退出码 2 报错，并列出认识的层名。

**`--top N`**：每层最多报几条。`0`（缺省）表示全报。

**`--json`**：输出机器可读结果。

**`--rules PATH`**：指定契约文件。缺省按环境变量 `QTFOUNDER_CODE_CONTRACT`、再从当前目录往上找 `src/code/default-contract.toml` 或 `src/cli/src/code/default-contract.toml`。

### `reflect`：任务完成后的架构反思清单

**NAME**：只看某一条（按 `name`），缺省四条全打。名字打错以退出码 2 报错，并列出认识的 name。

**`--example`**：连 `example` 层一起打。缺省只给 description 与 why——要不要例子按需，缺省省掉。

**`--json`**：每条输出 `{name, description, why, example}` 四层，缺省全给（JSON 不缺例）。

`reflect` 不扫描代码、不产生命中，正常退出恒为 `0`；契约读不到、没有 `[[reflect]]`、name 不认识，以退出码 2 报错。`--json`／`--rules` 在缺省检查与 `reflect` 两处通用。

### `review`：意图审查骨架

**ROOT**：要审的目录或文件，缺省当前目录。目标不在以退出码 3 报错。

**`--layer`**：只取某几层的命中当锚点，写法同缺省检查。

**`--example`**：连六条纪律的 `example` 一起打。

输出四节骨架（结论／意图假设／偏移清单／建议）与六条交付纪律；偏移清单里是语用命中的 `file:line` 锚点，**不排序、不标严重度**——严重度对着意图判，归写报告的人。锚点计数进退出码：有命中退 `1`，零命中退 `0`。

JSON 输出 `{root, intent_layers, rules, findings}`；`intent_layers` 是五层反推提问，与 Skill `code-refactor` 的正向五层一字不差。

## 二、退出码

退出码 `0` 表示无命中，`1` 表示有命中，`2` 表示用不了——契约读不到、层名不认识或有文件解析失败。

**解析失败不影响其他文件**：坏文件在标准错误输出里点名，已成功的部分照常报，退出码最后置 2。

## 三、JSON 结构

```json
{
  "findings": [
    {
      "layer": "pragmatic",
      "layer_name": "语用层",
      "kind": "uninformative_name",
      "path": "src/code/mod.rs",
      "line": 183,
      "message": "「run」只说了动作没说对象：读者不知道它处理的是什么、结果长什么样",
      "principle": "违反质准则（不说没理由的话）",
      "advice": "把宾语写进名字，或直接改成说清结果的名字"
    }
  ]
}
```

字段含义：

- `layer` 是稳定标识，会进脚本；`layer_name` 是给人看的中文名，可能改词，别拿它做匹配。
- `kind` 是这条验收标准的标识，同一 `kind` 的 `message` 会带不同的具体内容。
- `line` 为 0 表示这条命中落在整个文件上，不是某一行。
- `principle` 说明违反 Grice 哪一条，这是与通用风格检查器的分界。

## 四、契约字段

`default-contract.toml` 分七段：`scope`／`lexical`／`syntax`／`module`／`pragmatic`／`reflect`／`review`。`scope.exclude` 定不审的目录，与 `language`、`docs` 两份契约对齐。数值型字段一律「`0` 表示关闭这项检查」，词表型字段空数组表示不查。

`[[reflect]]` 是数组表，每条必填 `name`／`description`／`why`，`example` 缺省空串（空则 `--example` 不打该层）。顺序即优先级。**它没有「`0` 表示关闭」的约定**——空数组表示契约里没写反思清单，`code reflect` 报错，不静默空转。

改契约立即生效，不需重编译。这一点有测试兜底：`a_changed_contract_changes_the_verdict`。

## 五、稳定性承诺

`--json` 的字段名与 `kind` 取值在同一个大版本内不变。中文 `message`／`advice` 的措辞会随验收标准打磨而改，**不要对它做断言**。

`layer` 的四个取值不变；加层意味着大版本。
