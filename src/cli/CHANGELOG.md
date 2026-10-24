# Changelog

## [Unreleased]

### Added

- `qtfounder audit`：合并 `language` 与 `docs` 两套审计，按「一个词、一条规则」归堆，每期只报一处最值得改的。术语优先，目录级体裁要求与三类误报不进报告；`--json` 出结构化的数

### Changed

- `docs` 模块的 `contract`、`model`、`scan` 改为 `pub`，供 `audit` 直接复用扫描

## [0.0.9] - 2026-10-24

`health` 并进 `think`，认知模型抽成契约。

### Changed

- `qtfounder health` → `qtfounder think load`，模块从 `src/health/` 迁到 `src/think/load/`；提示词与用户级目录路径同步
- 认知模型从代码搬到 `src/think/default-contract.toml`：认知扭曲词表、归属与控制取值域、栏位顺序、追问用语与中文标签都在里面，判定层只读它
- 模型查找顺序：`--model` → `QTFOUNDER_THINK_MODEL` → 源码树 → `~/.config/qtfounder/think/model.toml` → 内置兜底

### Added

- `think model`：把当前生效的模型摊开，`--json` 给机器看
- `reflect --model <PATH>`：临时换一份模型
- 文档 `user-guide/think.md` 与 `api-reference/think.md` 扩成三个子命令

## [0.0.8] - 2026-10-24

`reflect` 从 `health` 挪到新的 `think` 组：检查器与建模分开。

### Changed

- `health reflect` → `think reflect`。`health` 回到纯语料诊断，不带子命令
- 数据落点由 `<数据目录>/reflect.json` 改为 `<数据目录>/think/reflect.json`
- 文档两篇由 `user-guide/health.md`、`api-reference/health.md` 改为 `think.md`

## [0.0.7] - 2026-10-24

`health` 加子命令 `reflect`：一条反思的 CBT 记录。

### Added

- `health reflect`：情境、自动思维、认知扭曲、正反证据、替代思维，加一个能回填的行为实验；角色归属接 `founder-reflect` 的两问。四个入口互斥：`--stdin` 记一条、`--list`（`--due` 只看到期）、`--id` 看一条、`--observe` 回填结果
- 记完返回 `missing` 与 `next_question`，按填栏顺序给出下一问；`recurring` 只列出现两次以上的认知扭曲
- 认知扭曲词表十项，中英文互为别名，落库统一存中文
- 数据落 `$XDG_DATA_HOME/qtfounder/reflect.json`，`QTFOUNDER_DATA` 覆盖
- 文档两篇：`user-guide/health.md`、`api-reference/health.md`，目录与首页同步

### Changed

- `health` 从叶子命令改为可选子命令组；不带子命令仍是原来的会话语料分析，行为不变

## [0.0.6] - 2026-10-24

拿 `docs` 查自己的文档站，清掉全部命中。

### Changed

- 三处退出码列表改为散文：`api-reference/` 下的 `code`、`docs`、`language` 三篇
- 一处验收标准分类改为散文：`user-guide/code.md`；`dev-guide/adding-a-check.md` 的四处清单同样并成段落
- 两处超长列表项拆开：`dev-guide/roadmap.md` 的阈值标定项、`dev-guide/docs/roadmap.md` 的阶段一步骤
- `dev-guide/index.md` 去掉一处三十四字的整句加粗
- 开发指南路线图的当前版本与已落地项更新到 `cli/v0.0.6`

## [0.0.5] - 2026-10-24

`docs` 加两层：加粗与分割线。

### Added

- `docs` 加两层：**加粗**（每千字处数、一行并列处数、单个加粗字数，短于三百字不算密度）与**分割线**（全文上限与紧挨标题的装饰线）；`--kind` 新增 `bold`、`rule`。契约加 `[bold]`、`[rule]` 两段
- 文档站补两层说明与契约字段，开发指南路线图加「阶段四」

### Fixed

- 加粗层不把表格行算进去：表头加粗是表格自己的写法，不是正文在标重点

## [0.0.4] - 2026-10-24

修 `docs check` 在子模块里的误报，同步一批禁用词。

### Changed

- `language` 契约补禁用词 `记账`、`账目`、`两笔账`、`第一笔账`、`第二笔账`：2026-10-24 用户要求非财务语境不用记账类术语；`账本`、`对账`、`口径` 已在 2026-10-22 批次内

### Fixed

- `docs check`：技能目录在祖先仓时（如子仓 `data/context`）只校验本仓的引用是否失效，不再要求子仓把祖先仓的全部技能写进索引；输出里标出「在祖先仓」与索引归属

## [0.0.3] - 2026-10-24

第四个子命令 `docs` 上线：文档结构审计。表格层从 `language` 整层迁出，再加列表与密度两层。

### Added

- CLI `docs` 子命令：表格、列表、密度三层，判这段内容该不该是这个形状。表格层自 `language` 迁入（`src/docs/table.rs`、`[table]` 契约、`Kind::Table`），行为不变；列表层查单项过长与三段式（`- **要点**：解释` 占比过高）；密度层查结构块占比与无散文率，只在缺陷规则未命中时兜底，严重度固定为嫌疑。契约在 `src/cli/src/docs/default-contract.toml`，查找顺序与 `code`／`language` 一致
- `docs check`：比对 `.agents/skills/` 实际目录与 `AGENTS.md`、`CONTRIBUTING.md` 里的引用，报缺失索引与失效引用
- `--stats`：只打每篇的块占比、无散文率、结构块数与字数，不产生命中；密度阈值标定前用它看分布
- 文档三篇：`user-guide/docs.md`、`api-reference/docs.md`、`dev-guide/docs/`（定位与路线图），`README.md` 与文档站目录同步

### Changed

- `language` 移除表格层：`Kind::Table`、`[table]` 契约段与 `table.rs` 一并移出，`--kind` 剩六个文字层。拆分不留兼容入口，避免两套口径走开
- 文档审计路线图四项（`docs` 子命令、列表层、密度兜底、`docs check`）随本版落地，只剩阈值标定

## [0.0.2] - 2026-10-22

第三个子命令 `health` 上线：读会话库看消耗落在哪，每条命中挂一句原话。它不做情绪画像。

### Added

- CLI `health` 子命令：读 Hermes 会话库（只读、只取 `role='user'`、同会话同内容去重），
  判两类消耗信号（`fatigue` 真的累 / `friction` 烦与返工）、来源短语与责任环节，
  每条来源必须挂一句原话，没有原话不计命中。口径在 `src/cli/src/health/prompt.md`。
  与 v0.0.1 移除的旧 `health`（check/track/history/profile，从 fiction+memory 的 git 历史
  提情绪维度）不是同一件事——**情绪画像仍归 memory 五轴档案，本命令不做情绪画像**，
  只回答「这股消耗来自哪里、落在助手哪个环节」。责任口径定死为「全部在助手一侧」，
  按 A–E 五个环节归位，不设「用户」「双方」「不明」选项。凭据沿用 language 的语义层
  （`LLM_API_KEY` → `~/.pi/agent/auth.json` → `~/.bashrc` 的 `DEEPSEEK_API_KEY`），
  `temperature=0`、只收 JSON。新增依赖 `rusqlite`（bundled）读会话库
- 判定口径可外置：`--prompt` → `QTFOUNDER_HEALTH_PROMPT` → 源码树 → `~/.config/qtfounder/health/`，
  四道都没有就报错。与 `code`／`language` 的契约同规矩，装成 CLI 之后脱离源码树也能跑
- `docs/user-guide/health.md`：命令、参数、退出码、判定口径、五环节、凭据、隐私边界、已知偏差

### Fixed

- 会话库读取：`messages.id` 与 `timestamp` 按 `cast` 取。原来按 text/real 直取会
  `Invalid column type` 直接失败——真机上跑过一次才暴露

## [0.0.1] - 2026-10-22

`src/cli` 从脚手架变成能用的工具：两个子命令查两类介质，每个介质的验收标准写成一份可在运行时替换的契约。

### Added

- CLI `code` 子命令：代码语用检查四层（词汇／句法／语篇／语用）。解析与判定分开——判定层只读 `ir::Facts`；Rust 走 `syn`、Python 走 `tree-sitter`，Go／TypeScript／Dart 共用一份通用提取器、差异全在 `specs.rs` 的一份 `LangSpec` 里，加语言只写配置。五语言一致性由测试守住。阈值在 `src/cli/src/code/default-contract.toml`——一份磁盘上的软件工程契约，运行时按 `--rules`／`QTFOUNDER_CODE_CONTRACT`／逐级往上找的顺序定位，**不内置兜底**，找不到就报错。`unused_imports` 刻意不判——rustc 已权威负责
- CLI `language` 子命令：文字审计七层（落点／自造术语／体系词／数字／句法／表格／语用），契约在 `src/cli/src/language/default-contract.toml`。带 `--sense` 语义层时只把规则挑出的段送模型，`temperature=0`、只收 JSON、直连 HTTP 不依赖内部 SDK；判不了归人。与迁移前的 Python 实现实测对等：同一份语料、两种扫描范围下命中数与按类分布完全相同
- `docs/` 三组文档：用户指南两篇（`code`／`language`，以自举方式介绍作用——拿工具查它自己的源码与文档）、开发指南按模块分文件夹（通用四篇 + `code/` + `language/`）、接口参考两篇（参数／退出码／JSON／稳定性承诺）

### Removed

- CLI `health` 子命令（check/track/history/profile）：默认路径随 `docs/` → `assets/` 迁移失效、无使用痕迹，情绪画像归 memory 五轴档案（`profile/emotions.md`）维护
