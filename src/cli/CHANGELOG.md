# Changelog

## [Unreleased]

### Added

- `design-review` Skill 随 CLI 发布（第八条）：交互设计评判十二条——先立问题再做屏幕、减分项七条、交付纪律三条，配评分标准；原则自 qtcloud 仓的交互设计评判原则抽取，案例存照随文。`qtfounder design principles` 摊开全文
- `qtfounder design review`：界面交付物走查，只做机器判得了的两条的数字面——第九条中文微排版（`font-size` 低于 12px 报缺陷、12–14px 报嫌疑，rem 按 16px 根换算，em 只计数不换算）与第五条主行动唯一（class 含 `primary` 的按钮一文件多于一个报一次嫌疑）。目录递归扫 `.html`，跳过 `.git`／`node_modules`／`target`／`_build`；其余十条归 Skill 里的走查与自评分
- `docs-format` Skill 随 CLI 发布（第七条）：照《量潮科技文档格式章程》管正式 Markdown 文档的排版与结构，提交前跑 `qtfounder docs format <文件>` 看格式状态与审计命中。章程原文两处用词按禁用词清单改写（`制定依据` → `制定理由`、`条目` → `短条`），条文含义不变
- `qtfounder docs format`：格式状态 + 审计。先数一篇（或一目录）的格式元素——标题（含一级个数与最深层级）、表格张数与行数、列表块数与项数、加粗处数、分割线条数、块引用段数、代码块个数与正文字数，目录模式按格式元素数从多到少排；再跑同一套审计。七类元素怎么算照《量潮科技文档格式章程》第四条，计数与审计共用同一处识别。`--json` 多一组 `files`

### Changed

- `docs/dev-guide/roadmap.md` 迁至路线图仓 `data/roadmap/qtfounder/roadmap.md`：方向文档统一归路线图仓，仓内只留开发指南；新增「qtcloud 组织验证」一节（探底噪、真实交付对照、漏率刻度三步，先记顺序不实现）。`docs/index.md`、`dev-guide/index.md` 与 `myst.yml` 目录同步摘除
- `docs` 的 `--kind`／`--rules`／`--top`／`--json`／`--stats` 改为全局参数：`qtfounder docs format docs --json` 这样写在子命令后也认
- `--kind` 报错提示补上 `journal`：七个取值本来就含它，提示里却漏了

- `meta validate` 改为契约自检：只查契约内部一致（重复名字、悬空引用、落空路径），不再读任何仓库的镜像文档，移除 `--docs`；镜像只给人看，机器只认契约

## [0.0.41] - 2026-10-27

### Added

- `meta category` 增加两端：`context`（入口）与 `archive`（出口）排在十八格之后，`meta category context` 可直接查；收什么写在契约 `[[asset]]` 的 `description`
- `meta validate` 同时比对两端：称呼、目录与界线不一致都报漂移

### Changed

- 契约 `Asset` 新增 `description`（缺省空）；`Contract::asset` 按名字取资产

## [0.0.40] - 2026-10-27

### Changed

- 移除归位契约里已停用的 `[maintain]` 九条维护规则与对应结构体；`asset maintain` 只按资产关系工作

## [0.0.39] - 2026-10-27

### Changed

- `asset maintain` 按「领域 / 资产」结构重写：整理 context、校验、按关系搬到长期记忆仓；`--check` 只校验，`--dry-run` 预览，`--json` 给脚本读；目标已存在不覆盖
- `asset catalog` 改为 20 种资产，路径精确到仓库；`asset relation` 改为按角色与迁入边输出
- 契约新增 `[[asset]]` 20 条与 `[[relation]]` 3 条；toolkit、platform、laboratory 的 path 修正为具体仓库

## [0.0.38] - 2026-10-27

### Added

- `asset maintain`：资产维护视图——九条共用做法（目录、索引、变更记录、版本、发布、提交、迁移、闸门、只读），规则写在归位契约的 `[maintain]` 段；`--json` 给脚本读

## [0.0.37] - 2026-10-27

### Added

- `meta domain` 改为本体论视图：域 → 范畴、属性与公理；每个命令的契约顶层新增 `domain`，标明它属于哪个域
- `meta home`：看去处——内容搬出本仓能去哪里
- `meta validate` 纳入 `home.md` 与模型的轴（`category.md` 里的 `- 轴：`）

### Changed

- 契约的 `[[domain]]` 改为 `[[home]]`；卡片字段 `domain` 改为 `home`，`min_domain_hits` 改为 `min_home_hits`
- 公理 `domain-before-category` 改为 `home-before-category`，表述改为「先定去处，再定范畴」
- `docs/gallery/meta/domain.md` 改为本体论说明，原六个领域搬到 `home.md`

## [0.0.36] - 2026-10-27

### Changed

- 契约的 `[[cell]]` 改为 `[[category]]`，结构体 `Cell` 改为 `Category`；`paradigm.source` 取值由 `cells` 改为 `categories`
- 公理 `domain-before-cell` 改名为 `domain-before-category`
- `classify` 与 `relation` 的 JSON 字段 `cell` 改为 `category`

## [0.0.35] - 2026-10-27

### Added

- `meta validate` 纳入 `category.md` 与 `paradigm.md`：名字、族、目录、界线、状态、来源、合成、对象与态射数逐条比，缺页也报
- 文档新增 `docs/gallery/meta/category.md`（十八格）与 `paradigm.md`（两个范式）

## [0.0.34] - 2026-10-27

### Added

- `meta paradigm`：把归位写成一个范式——对象、态射、合成律；`source = "cells"` 直接取格与降格，别的范式写在 `[[paradigm]]` 里，不另存一份
- `meta paradigm --check`：查合成缺口与互相可达（同构嫌疑）；`compose = "transitive"` 缺合成或任一范式有互达时退出码 1
- `meta compare <左> <右>`：比两个范式的同名、同称呼、各自独有与形状，列出可吸收候选
- 契约新增亚里士多德十范畴，作为第二个范式

## [0.0.33] - 2026-10-27

### Changed

- 六个领域的 `status` 全部置为 `confirmed`；`domain.md` 同步标记「确认」

## [0.0.32] - 2026-10-27

### Added

- `meta domain`：看登记的领域——名字、称呼、线索与确认状态，确认的排前；契约的 `[[domain]]` 新增 `status`
- `meta validate` 纳入 `domain.md`：名字与状态不一致都报

## [0.0.31] - 2026-10-27

### Added

- `meta property`：看标准属性——字段名、所属组与定义，组内确认的排前；十九条定义进契约的 `[[property]]`
- `meta category`：看十八格各收什么，按族分组；与动词 `classify` 一静一动
- `meta validate`：查 `axioms.md`、`property.md` 与契约是否还对得上——名字、状态、分组、定义任一处不一致就报，对不上时退出码 1

## [0.0.30] - 2026-10-27

### Changed

- 确认过的公理优先：`meta axiom` 把 `confirmed` 排在前面，同组内保持契约顺序；落位卡的公理行同样先列确认的，并在每条后标出确认或怀疑。公理仍不参与算分

## [0.0.29] - 2026-10-27

### Changed

- `statement`、`summary`、`boundary` 三种内容字段统一成 `description`：公理、模型、族、格各用一份描述。`[[axiom]]`、`[model]`、`[[model.family]]`、`[[cell]]` 的字段与 `meta axiom`、`meta model`、`asset contract` 的 JSON 键一并改；`why`、`skeleton`、`cues` 不动，它们不是描述

## [0.0.28] - 2026-10-27

### Changed

- 格不再用 `key`：机器可读标识并入 `name`，原来的中文名改用 `label`。`[[cell]]` 的字段、`meta classify` 与 `asset *` 的 JSON 键（`key` → `name`、`name` → `label`）、`asset contract <name>` 与 `asset relation <name>` 的位置参数一并改；与公理、领域、家族的名字口径一致

## [0.0.27] - 2026-10-27

### Changed

- 公理的 `id` 改为 `name`：uuid 留给 id，name 做机器可读标识。`[[axiom]]` 字段、`meta axiom` 的 JSON 键（`id` → `name`）、报错文字与文档口径一并改；读旧契约会在解析时报缺 `name`

## [0.0.26] - 2026-10-27

### Added

- 公理多了 `status`：`confirmed` 或 `suspected`，缺省按怀疑算。`meta axiom` 在 id 后打「确认」或「怀疑」，JSON 带 `status` 与 `status_label`，写了别的值在读契约时拦下。现只有 `template-scope` 标确认，其余八条标怀疑

## [0.0.25] - 2026-10-27

### Changed

- 修正 `template-scope` 的前提：模板跟固定结构走，不跟目录走。旧说法「模板只归 `docs/` 下的手册与规格，日志可以有日期模板」自相矛盾——日志在 `data/journal/`，不在 `docs/` 下；日志有日期模板的真正理由是它的输入格式固定

## [0.0.24] - 2026-10-27

### Added

- `meta classify` 的落位卡新增 `公理` 一行：只点名这次判断真用到的公理（定域点 `text-only`、指了域名点 `archive-vs-transfer`、收了点 `one-place` 与 `template-scope`、不收点 `no-fit`），JSON 同步多了 `axioms` 字段。公理不参与算分，只留一行出处；契约里删掉的公理不再上卡

## [0.0.23] - 2026-10-27

### Fixed

- 公理的 `statement` 与 `why` 原来只是字段：空着也只有内置契约的单测会报，用户级契约删掉 `why` 系统不吭声。现在两者为空都会在读契约时拦下，与 `meta` 用户文档的说法一致

## [0.0.22] - 2026-10-27

### Added

- `qtfounder meta model`：看两族九宫格的轴与格，以及登记过的领域
- `qtfounder meta axiom`：看归位时不重新论证的前提；契约新增 `[model]` 与 `[[axiom]]`，模型家族与 `family` 对不上、公理 id 重复或缺 `why` 都会在读契约时拦下

## [0.0.21] - 2026-10-27

### Changed

- Agent 契约的 `cli_scopes` 补齐 `asset`、`git`、`product`，去掉不是命令组的 `skill`；`agent relation` 的覆盖统计随之准确

## [0.0.20] - 2026-10-27

### Added

- `qtfounder asset`：`contract` 看十八类资产的边界、分档与写法，`catalog` 看仓库里在不在，`relation` 看家族与目录分组
- `qtfounder meta relation`：看一格的迁出、迁入与分辨句
- `qtfounder agent relation`：看 Skill 引用了哪些命令，以及哪些命令组还没有 Skill

## [0.0.19] - 2026-10-27

### Added

- `qtfounder docs` 新增日志模板校验：日志文件名用 `YYYY-MM-DD.md`，写了标题就必须是 `# YYYY-MM-DD` 且与文件名一致；不写标题放行。日志目录仍跳过通用格式规则，这一层单独跑

## [0.0.18] - 2026-10-27

### Changed

- 洞察不再规定写法：`meta` 契约的骨架去掉章节表，只留「要交代什么」；`write` 的 `insight` 体裁去掉 required 与 `min_h2`，只留禁区。固定模板归 `docs/` 下的手册与规格，陈述型不查标题顺序

## [0.0.17] - 2026-10-27

### Added

- `qtfounder git hook`：`install`／`status`／`uninstall` 管主仓与各层子模块的 `core.hooksPath`；只改配置，不动脚本，指向别处默认不覆盖
- `qtfounder git hook run`：钩子脚本的执行入口；`pre-commit` 只拦新引入的用词错误，`post-merge` 与 `post-checkout` 对齐子模块指针
- `qtfounder git submodule check`：查子模块的指针、未提交改动与推送状态；`--pushed` 只查会挡推送的几项
- `pre-push` 钩子：推父仓前跑 `submodule check --pushed`，挡住不在远端的子模块指针

### Changed

- `meta` 契约的洞察写法改为成篇体例：背景与问题、判断、机制、边界、结论；不再按已确认/假说分节，可信度写进「边界」
- `write` 契约的 `insight` 体裁改认「判断／机制／结论」任一，`min_h2` 从 1 提到 3；原来的「假说／已确认」要求会把人逼回分节写法

## [0.0.16] - 2026-10-25

自 0.0.13 起累积的变更；0.0.14 与 0.0.15 未单独发版。

### Added

- `qtfounder write`：表达审计——体裁、读者、主题三轴，加空话、口号、指路、结构四层；契约落 `src/write/default-contract.toml`，`--profiles` 列出可用画像
- `write-express` Skill：把口号、类比与结论堆叠改写成可读文档；先跑 `qtfounder write`，再按体裁、定义、步骤、边界与验收重排
- `qtfounder product organize`：按 `actual` 归一化键把全部记录聚成主题，并给出 `rules.md` 的规则原文，供归档使用
- `qtfounder product clear --id <clr_id>`：删除已归档的澄清记录，清空的需求一并移除，`rules.md` 不动
- `product` 组升进正式 CLI：`requirement` 与 `clarify` 两个入口从实验室搬来，数据落 `$XDG_DATA_HOME/qtfounder`（未设则 `~/.local/share/qtfounder`），环境变量 `QTFOUNDER_DATA` 可换目录；新增 `user-guide/product.md` 与 `api-reference/product.md`
- `meta classify` 的洞察线索补上工程类机制词：前提、地基、断言、否则、返工。讲「为什么必须这样排」的归洞察，讲「现在做哪一步」的仍归路线图
- 契约新增 `min_domain_hits`：领域命中数不到下限不指路，一个顺带出现的领域词不再把整篇搬去别的领域

### Changed

- `docs` 的 frontmatter 契约接上 `genre`／`audience`／`topic`：三张取值表与 `write` 契约的画像对齐，写错报错
- `write` 契约改为先看体裁、读者、主题：新增 `[genre.*]`、`[audience.*]`、`[topic.*]` 与 `--genre`／`--audience`／`--topic`／`--profiles`；句子形态四层保留为通用检查
- `agent` 契约登记 `write` 范围：新增 Skill `write-express`，写法检查要求正文出现 `qtfounder write`
- `product-clarify` 改名 `product-collect`，`product-requirement` 改名 `product-organize` 并随 CLI 发布；现在嵌六条 Skill，`product-organize` 用 `product organize` 与 `product clear` 完成归档与清缓存
- `product-clarify` Skill 指向 `qtfounder product`，不再依赖 `qtfounder-lab`；前置从「装实验室 CLI」改为「查正式 CLI」
- 实验室 CLI 整体删除：`health` 早已由 `think reflect` 取代，`product` 升进正式版；`examples/quanttide-founder-lab/src/cli` 不留复制件
- 归位契约：实验室的边界补上「验证通过就升进正式入口，不留复制件」，新增实验室/平台的分辨句；`meta-classify` Skill 同步这条规则
- `think` 的宽筛词表进模型：46 个弱信号词、16 个纯操作短句与长度门槛从 `load/screen.rs` 移进 `[screen]`，改词表不用重编译；`think model` 的输出也摊开这一节
- `code` 的两张词表进契约：无信息函数名与错误信息里的行动词从 `pragmatic.rs` 移进 `[pragmatic]`；跳过目录从 `mod.rs` 移进新增的 `[scope].exclude`，与另外两份契约对齐
- `meta-classify` Skill 从 389 行收到 200 行左右：十八格表、各格写法、准入、分级、语义界线与正反例移出，格子数据与骨架以契约为准，例子进 CLI 测试；正文只留读卡、定域与迁移、日志分诊、自检与误判
- 契约补「洞察/愿景」「愿景/档案」两对分辨句，替代 Skill 的语义界线
- 平台线索去掉「系统」：这个词到处出现，会把自指句误判成平台
- 洞察与路线图的分辨句改为「在解释为什么必须这样排」与「现在就要做哪一步」二选一
- `meta-classify` Skill 的语义界线与正反例同步这条判断

## [0.0.13] - 2026-10-25

### Added

- `product-clarify` 随 CLI 发布：现在嵌四条 Skill，`list`／`install`／`check`／`lint` 的用法不变

### Changed

- 该 Skill 里的 `code clarify` 改为 `product clarify`，与 `qtfounder-lab product` 的实际命令对齐

## [0.0.12] - 2026-10-25

`agent` 组：装 Skill，查写法。

### Added

- `qtfounder agent skill`：一组四个入口。`list` 与 `check` 看装到哪一步，`install` 装一条或全部（`--copy`、`--claude`、`--force`、`--root`），`lint` 查名字、说明与正文
- `lint` 查七样：目录名与 `name` 一致、小写连字符与分段、登记过的范围、动词原形、说明写触发时机、正文有标题、命令组名字要点名对应命令；另查跨文件重名。标准未定，逻辑先留在 CLI，不进工具箱
- 随 CLI 发布三条 Skill：`meta-classify`、`language-govern`、`think-reflect`。它们的正文点名的命令都由本 CLI 提供，必须同版本
- 清单在构建时从包内 `skills/` 嵌入，可用 `QTFOUNDER_SKILLS_DIR` 换来源；目录为空时清单为空，仍能构建
- 文档合并成 `user-guide/agent.md` 与 `api-reference/agent.md`

### Changed

- 安装命令从顶层 `skill` 并进 `agent skill`：`skill list/install/check` 不再单独存在，装了 CLI 只有一组入口
- 安装位置缺省是系统位置（`$HOME/.agents/skills`），不装进当前仓；只装某个仓时给 `--root`。去掉 `--global`：它现在是默认行为
- 安装实现从 `src/skill/` 移进 `src/agent/`：`install` 管装，`registry` 管嵌入清单，`lint` 管检查
- 依赖工具箱的 `agent::skill` 模块：从 crates.io 取 `quanttide-founder`；嵌装逻辑在库里，CLI 只登记清单与命令行
- 跨域提示与用户指南里的 Skill 名改为 `meta-classify`：原来的 `meta-domain` 与 `meta-category` 已合并

## [0.0.11] - 2026-10-25

### Fixed

- `meta classify --help` 把「不给路径就读标准输入」改成与实际一致：不给路径要用 `--stdin`

## [0.0.10] - 2026-10-25

`meta` 组落位：判一段内容该进哪一格、要不要跨域。

### Added

- `qtfounder meta classify`：给一段内容一张落位卡——域（留在这里／去哪个领域仓，移交还是归档）、格（十八格之一与目录）、档（已确认／假说、已决策／待决策候选等）、写法（该格的骨架）。判不了时只问一句，套不上就不收，整篇材料先要求拆条
- 归位契约 `src/meta/default-contract.toml`：十八格、六个领域的线索、降格规则与分辨句都在文件里，判定层只读它；`from`／`to`／`pair` 指向不存在的格在读契约时就报错
- 退出码新增 3（待定）：脚本可以停下来等人回答那一问，不必当成失败重跑
- 文档 `user-guide/meta.md` 与 `api-reference/meta.md`，目录与首页同步
- 语言检查认语境声明：文件头写 `context: finance`，契约里标了同一语境的词（记账、账目、账本、对账、口径与两笔账类）就不再报。没声明的文件照报——例外要说出来，不能靠读的人猜
- `qtfounder docs` 新增 front matter 层：字段名不在契约表里报嫌疑，`status` 与署名 `type` 的取值不在表里报错；没有 front matter 的文档不动
- front matter 的键表收入 MyST 保留字段（`title`、`authors`、`exports` 等），站点渲染用的字段一律放行；缩进值与列表项不当字段读
- `qtfounder audit`：合并 `language` 与 `docs` 两套审计，按「一个词、一条规则」归堆，每期只报一处最值得改的。术语优先，目录级体裁要求与三类误报不进报告；`--json` 出结构化的数
- `audit --patrol`：存量巡检。逐仓抽历史提交（默认 40 个，两端必取、中间等距），导出成树后用今天的契约重判，再和今天逐条比，分「随文件消失」「在留存文件里改掉」「今天还在」三档；`--samples` 调抽样深度

### Changed

- `language` 契约的禁用词新增 `contexts`：写清词属于哪个语场，file 声明同一语境时放行；目前只标了财务一族
- `docs` 契约新增 `[frontmatter]`：`keys` 定字段名，`status` 定三档，`author_types` 定署名类型，`orgs` 定法人全名，`contexts` 定语境取值；`authors` 每条必须标 `type`，`agent` 写模型名，`org` 只认四个全名
- `docs` 模块的 `contract`、`model`、`scan` 改为 `pub`，供 `audit` 直接复用扫描
- 契约查找多认一条仓内路径 `apps/qtfounder/src/cli/src/...`：在主仓根跑 `audit` 时不再落到 `~/.config` 的旧文件（缺 10-24 新增的「记账」类词条，体检少报 13 处）
- `docs` 契约补上 `data/archive`、`data/journal`、`assets`、`examples` 四个排除项，与 `language` 契约对齐：格式类错误从 620 处降到 54 处，嫌疑从 787 降到 340

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
