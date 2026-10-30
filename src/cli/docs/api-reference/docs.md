# `qtfounder docs` 接口

## 一、命令面

```
qtfounder docs [ROOT] [--kind KIND] [--rules PATH] [--top N] [--json] [--stats]
qtfounder docs format [ROOT] [--kind KIND] [--rules PATH] [--top N] [--json] [--stats]
qtfounder docs check [ROOT]
```

**ROOT**：目录或单文件，缺省当前目录。递归找 `.md`。

上面五个选项是全局的：写在 `format`／`check` 前后都认，例如 `qtfounder docs format docs --json`。

**`--kind`**：`table`／`list`／`density`／`bold`／`rule`／`frontmatter`／`journal`，只报一层。`format` 模式下只过滤审计，状态始终全数。

**`--rules PATH`**：你给了就用它指的契约。不给就按 `QTFOUNDER_DOCS_CONTRACT`、再往上找 `src/docs/default-contract.toml`、最后退到 `~/.config/qtfounder/docs/`。

**`--top N`**：最多列几个文件，缺省 15。`--stats` 时也用它限制行数。

**`--stats`**：打印密度数字，不产生命中。阈值标定前用它看分布。

**`docs format [ROOT]`**：先出格式状态（每篇的格式元素计数），再出同一套审计。目录模式按 `elements` 从多到少排，`--top` 限行。

**`docs check [ROOT]`**：比对 `.agents/skills/` 实际目录与 `AGENTS.md`、`CONTRIBUTING.md` 里的引用，不扫正文。

## 二、退出码

退出码 `0` 是没命中；对 `docs check` 而言，是索引与目录对得上。`1` 表示有命中；对 `check` 而言，是存在缺失索引或失效引用。

`2` 表示用不了：契约读不到、`--kind` 取值不认识、有文件读不了，或 `check` 的目标目录不存在。

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

`severity` 只有 `error` 与 `suspect` 两值：表格、列表、加粗、分割线命中是 `error`，密度、加粗密度与日志模板是 `suspect`。`excerpt` 最多七十字符，用来定位，不要拿来做断言。

`docs format --json` 多一组 `files`，每篇一行计数，`findings` 与上面同构：

```json
{
  "scanned_files": 16,
  "files": [
    {
      "path": "user-guide/docs.md",
      "chars": 1800,
      "headings": 7,
      "h1": 1,
      "deepest": 2,
      "tables": 0,
      "table_rows": 0,
      "lists": 0,
      "list_items": 0,
      "bold": 12,
      "rules": 0,
      "quotes": 0,
      "code_blocks": 2,
      "elements": 21
    }
  ],
  "findings": []
}
```

**字段怎么算**：`chars` 是 frontmatter 与代码块之外的非空行字数（含标题与表格行）；`headings` 只数 ATX 标题（`#` 到 `######`），`deepest` 为 0 表示没有标题；`tables` 只数连续两行以上 `|` 的块，`table_rows` 是这些表的总行数；`lists` 把只隔空行的松散列表算同一块；`bold` 不含表头加粗；`rules` 不含 frontmatter、代码块与 setext 下划线；`quotes` 按连续 `>` 行算段；`elements` = `headings + tables + lists + bold + rules + quotes + code_blocks`，这七类怎么算照《量潮科技文档格式章程》第四条。计数与审计共用同一处识别，状态与命中的数字不会对不上。

## 四、契约字段

`[table]` 沿用原 `language` 的表格层：`max_columns`、`max_cell_chars`、`max_rows`、`require_header`、`flag_two_column`、`min_rows`、`sparse_ratio`、`layout_headers`。

`[list]` 三个字段：`max_item_chars`（单项字数上限）、`label_ratio`（三段式占比上限）、`min_items`（少于这个项数不算结构块）。

`[density]` 四个字段：`max_block_ratio`（结构块字数占比上限）、`max_run_ratio`（无过渡段落的块占比上限）、`min_paragraph_chars`（多长才算过渡段）、`min_chars`（文档低于这个字数不判）。前两个默认 0，即关闭。

`[bold]` 四个字段：`max_per_1000`（每千字加粗处数）、`max_per_line`（一行并列加粗处数）、`max_span_chars`（单个加粗的字数）、`min_chars`（低于这个字数不算密度）。

`[rule]` 两个字段：`max`（全文分割线处数）与 `flag_between_sections`（紧挨标题的横线算装饰，单独报）。

`[frontmatter]` 各字段各管一张表，写在表外就报：`keys` 收允许出现的字段名（表外报嫌疑），`status` 收取值，`author_types` 收 `type` 的取值，`orgs` 收法人全名，`contexts` 收语境声明。

`genre`、`audience`、`topic` 三张表收表达审计要用的画像。

某张表留空就是不查它；文档压根没有 front matter 时整层不动。`keys` 默认已含 MyST 保留字段（`title`、`authors`、`license`、`exports` 等），站点渲染字段一律放行；本仓自用的是 `name`、`status`、`context`、`genre`、`audience`、`topic`。

署名读 `authors` 的块列表，只看每条内部的 `type` 与有没有 `name`：`type` 缺失报错，`agent` 缺模型名报嫌疑，`org` 的名字必须在 `orgs` 里；其他字段（`email`、`affiliations` 等）不查。流式写法（`authors: [...]`）报嫌疑，因为落不了行号。默认取值：`status` 为 `draft`、`pending`、`final`，`author_types` 为 `human`、`agent`、`org`；画像取值与 `write` 契约的 `[genre.*]`、`[audience.*]`、`[topic.*]` 对齐。

所有数值 0 都表示关闭，与 `language` 一致。

## 五、稳定性承诺

`[journal]` 的字段：`enabled`（开关）、`paths`（路径里出现这些片段即按日志处理）、`title_must_be_date`（写了标题就必须是 `# YYYY-MM-DD`，不写放行）。日志目录通常在 `[scope] exclude` 里，这一层单独跑，不受排除影响；说明文件（README、CONTRIBUTING、CHANGELOG、index）不按模板查。

`kind` 的七个取值（`table`、`list`、`density`、`bold`、`rule`、`frontmatter`、`journal`）与 `severity` 的两个取值不变；`excerpt`、`advice` 的措辞会变，别做断言。新增一层算小版本，删改取值算大版本。`files` 只在 `format --json` 里出现，是新增不是改动；`files` 的字段只增不改名。
