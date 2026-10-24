# `qtfounder docs` 接口

## 一、命令面

```
qtfounder docs [ROOT] [--kind KIND] [--rules PATH] [--top N] [--json] [--stats]
qtfounder docs check [ROOT]
```

**ROOT**：目录或单文件，缺省当前目录。递归找 `.md`。

**`--kind`**：`table`／`list`／`density`／`bold`／`rule`／`frontmatter`，只报一层。

**`--rules PATH`**：指定契约。缺省按 `QTFOUNDER_DOCS_CONTRACT`、再往上找 `src/docs/default-contract.toml`、最后退到 `~/.config/qtfounder/docs/`。

**`--top N`**：最多列几个文件，缺省 15。`--stats` 时也用它限制行数。

**`--stats`**：打印密度数字，不产生命中。阈值标定前用它看分布。

**`docs check [ROOT]`**：比对 `.agents/skills/` 实际目录与 `AGENTS.md`、`CONTRIBUTING.md` 里的引用，不扫正文。

## 二、退出码

退出码 `0` 表示无命中，`docs check` 时表示索引与目录一致；`1` 表示有命中，`check` 时是存在缺失索引或失效引用；`2` 表示用不了——契约读不到、`--kind` 取值不认识、有文件读不了，或 `check` 的目标目录不存在。

## 三、JSON 结构

```json
{
  "scanned_files": 16,
  "findings": [
    {
      "kind": "list",
      "severity": "error",
      "path": "user-guide/code.md",
      "line": 21,
      "excerpt": "**真该改的**：`run` 这种只说动作、不说对象的函数名……",
      "advice": "3/3 项是「加粗要点 + 解释」三段式（上限 70%）：整段重写为散文"
    }
  ]
}
```

`severity` 只有 `error` 与 `suspect` 两值：表格、列表、加粗、分割线命中是 `error`，密度与加粗密度是 `suspect`。`excerpt` 最多七十字符，用来定位，不要拿来做断言。

## 四、契约字段

`[table]` 沿用原 `language` 的表格层：`max_columns`、`max_cell_chars`、`max_rows`、`require_header`、`flag_two_column`、`min_rows`、`sparse_ratio`、`layout_headers`。

`[list]` 三个字段：`max_item_chars`（单项字数上限）、`label_ratio`（三段式占比上限）、`min_items`（少于这个项数不算结构块）。

`[density]` 四个字段：`max_block_ratio`（结构块字数占比上限）、`max_run_ratio`（无过渡段落的块占比上限）、`min_paragraph_chars`（多长才算过渡段）、`min_chars`（文档低于这个字数不判）。前两个默认 0，即关闭。

`[bold]` 四个字段：`max_per_1000`（每千字加粗处数）、`max_per_line`（一行并列加粗处数）、`max_span_chars`（单个加粗的字数）、`min_chars`（低于这个字数不算密度）。

`[rule]` 两个字段：`max`（全文分割线处数）与 `flag_between_sections`（紧挨标题的横线算装饰，单独报）。

`[frontmatter]` 四个字段：`keys`（允许出现的字段名，表外的报嫌疑）、`status`（取值表，表外的报错）、`author_types`（`authors` 每条 `type` 的取值表）、`orgs`（法人署名可用的全名，不在表里报错）。表为空则不查该项；文档没有 front matter 时整层不动。`keys` 默认已含 MyST 的保留字段（`title`、`authors`、`license`、`exports` 等），站点渲染用的字段一律放行；本仓自用只有 `name`、`status` 两个。

署名读 `authors` 的块列表，只看每条内部的 `type` 与有没有 `name`：`type` 缺失报错，`agent` 缺模型名报嫌疑，`org` 的名字必须在 `orgs` 里；其他字段（`email`、`affiliations` 等）不查。流式写法（`authors: [...]`）报嫌疑，因为落不了行号。默认取值：`status` 为 `draft`、`pending`、`final`，`author_types` 为 `human`、`agent`、`org`。

所有数值 0 都表示关闭，与 `language` 一致。

## 五、稳定性承诺

`kind` 的六个取值（`table`、`list`、`density`、`bold`、`rule`、`frontmatter`）与 `severity` 的两个取值不变；`excerpt`、`advice` 的措辞会变，别做断言。新增一层算小版本，删改取值算大版本。
