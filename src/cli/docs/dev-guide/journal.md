# journal：列出与归档

状态：设计稿，未实现。`list` 与 `archive` 已定；`index`、`find`、`ask`、`talk`、`defer`、`digest` 先分析、不定案。

关系先说清：thera diary 是探路的——一组日记交互先在它那边跑通，可靠性问题（落点、进度、脱敏、容错）也先在它那边修过一轮；qtfounder 要泛化——把探明的路按本仓的约定收成通用能力，落点、凭据、契约、测试全走本仓既有的面。两边无运行时依赖：不调用、不共享配置与文件格式。

## 现状（2026-11-06 实况）

日志散在三处：

- 当天与新页：`data/context/default/journal/2026-11-06.md`（`data/context/fiction/journal/` 同构）
- 已进 journal 仓、没进月份目录的散页：`data/journal/<域>/` 根上共 15 页，分布在 default、infra、create 等域
- 归档位：`data/journal/<域>/<YYYY-MM>/`，已有 18 个月份目录

口径差异，只报告：AGENTS 快速索引写「当天放集根 `data/context/default/YYYY-MM-DD.md`」，实况已在 `journal/` 子目录；以谁为准由人定，本设计不替它改。

已有三处接线，本设计不碰：

- `docs audit` 的 `[journal]` 层查文件名 `YYYY-MM-DD.md` 与标题 `# YYYY-MM-DD`
- `qtfounder data` 的标注源 `{"kind":"journal"}` 按空行切段
- `git` 模块的跳过表把 `/journal/` 归进「不是给人反复读的」

`data/context` 与 `data/journal` 是两个子模块，也就是两个仓：归档一次等于一边删一边加，提交要走两仓加父仓指针的分层链。

## 从 thera diary 吸收的经验

1. **期间写法**：`2026-03` 整月、`2026-03-05` 单日、`a..b` 闭区间。`list` 与 `archive` 共用同一套解析，写错退出码 2 报清楚。
2. **列目录不调 LLM**：列目录是纯文件系统活，快，能进脚本。
3. **扫描只认 `YYYY-MM-DD.md`**：README、CONTRIBUTING、index 按名跳过。本仓 `src/docs/journal.rs` 的 `date_of` 已是同款规则，复用它，不造第二套。
4. **生成物落 XDG、正文只读**：本仓已有 `QTFOUNDER_DATA` → `$XDG_DATA_HOME/qtfounder`（`src/core/store.rs`）。索引、清单、对话记录、消化稿这类生成物都落这里，不进 git。
5. **批量单点失败不中断**：它的索引按篇容错、消化按天保留队列重试——`archive` 照此：单页移动失败（权限、目标占用）跳过继续，最后汇总失败清单，退出码非 0。
6. **进度强制刷 stdout**：它修过管道下块缓冲导致进度不可见（`[n/N]` 强制 flush）。批量归档的进度行照此写。
7. **动用户数据先确认**：它的仓内 AGENTS 写死「涉及用户数据（移动、删除、重命名）前必须先确认」。`archive` 默认只预览。
8. **测试不碰真实环境**：它修过测试往真实 cwd 写、测试发真网络。本仓用临时目录加注入桩的惯例已有，新模块照办。
9. **局限写在文档里**：它把「检索不认同义词」直说。已定的两个动词的局限（不读正文）写明，分析中的动词各自的局限写在分析里。

## 避免的教训

1. **生成物写进仓库**：它 2026-11-06 才把硬编码 `data/` 的导出、往真实 cwd 写的测试、默认落仓的对话记录统一改成 XDG（提交 `dbcb534`），是补丁不是设计。本设计第一天就不产生仓内文件。
2. **根目录碎片转嫁给用户**：它的日记根要 `--journal` 手工指，文档还得教「新日志在哪个根」。本仓碎片更甚（三处），解析必须原生覆盖三处，不加参数、不设环境变量。
3. **命令面铺开再砍**：它 0.2.0 一次删掉 auto-commit、doc-check、submodule-sync、workflow 四个命令。本设计的动词逐个过分析再定，不一次性铺开——这正是「先分析、不下结论」要防的事。
4. **日记原文进 LLM 前不脱敏**：它是出过内容才补的脱敏提交（`ae9e51c`，抽取 prompt 的示例与测试数据里带过真实日记内容）。分析中的 `ask`、`index` 若做，脱敏是前置条件，不是补丁。
5. **归档顺手改写**：迁移纪律是只移动不改写。`archive` 不改文件名、不改正文、不改域；发现结构不对（某页该不该属这个域）就停下来报，不替人改。

## 已定：list 与 archive

```
qtfounder journal list [--period 2026-11]          # 跨三处列出，不调 LLM
qtfounder journal archive [--period 2026-11] [-y]  # 散页归位；默认预览，-y 才动
```

解析顺序：`data/context/<域>/journal/`（当天与新页）→ `data/journal/<域>/` 根上散页 → `data/journal/<域>/<YYYY-MM>/`（已归档）。同一日期出现在多处时全部列出并标出处，不替人判断谁是正本。

`archive` 的移动分两类，都只移动不改写：

- 域根散页进月份目录：`data/journal/default/2026-11-05.md` → `data/journal/default/2026-11/2026-11-05.md`
- context 新页进 journal：`data/context/default/journal/2026-11-06.md` → `data/journal/default/2026-11/2026-11-06.md`，域从路径读（`context/<域>/…` → 该域）；路径里读不出域的列出来待人指认，不猜

纪律：

- 默认预览，逐行打印「从 → 到」，`-y` 才执行
- 目标已存在就跳过并报，不覆盖（同 `asset maintain` 的规矩）
- 移动完只打印分层提交清单（`data/context`、`data/journal`、父仓指针），提交仍走 git-commit 流程，命令不自动提交
- 幂等：第二遍跑零移动

## 分析中的动词（未定案）

每个三段：探路版做了什么（事实）→ 泛化到本仓要回答什么 → 悬着的问题。

### index — 离线卡片索引

探路版：本地小模型（ollama qwen3:0.6b）逐篇抽 JSONL 卡片（summary、mood、topics、events、keywords、decisions），按 `date|domain` 幂等增量，`--force` 全量重建；解析容忍模型前置废话与代码块，单篇失败不断批；进度 `[n/N]` 强制刷 stdout；索引默认落 XDG。

泛化到本仓要回答的：

- 走哪条 LLM 出口——本仓已有两条（`think` 的 judge 与 `language` 的 sense，同款凭据链 `LLM_API_KEY` → `~/.pi/agent/auth.json` → `~/.bashrc`，POST `{base}/chat/completions`），index 用同款还是独立配置
- 默认本地小模型还是云端——本仓现有两条 LLM 面都走 `LLM_*` 环境变量，没有「本地默认」的先例
- 卡片 schema 是否原样——与 `qtfounder data` 的标注（标签集、fact/decision 类）是两套还是共用
- 索引落点——`core/store.rs` 的 `QTFOUNDER_DATA` 已有位置

悬着：做不做、schema、默认模型。

### find — 决策事件检索

探路版：读 JSONL 索引，ASCII 按词、中文按二元组切词打分（命中的不同词项数），同分按日期降序；零模型、可离线；局限明写——不认同义词，「睡不着」匹配不到「失眠」。

泛化到本仓要回答的：

- 检索对象是卡片还是正文——依赖 `index` 先存在，还是独立一层（直接扫文件名与正文，不依赖 LLM）
- 扫描范围——三处碎片全算，复用 `list` 的解析
- 出处——命中带路径与日期；本仓同日期可能在三处并存

悬着：与 `index` 的依赖顺序、检索层要不要独立成一层。

### ask — 对照两期提问

探路版：`--a/--b` 注入两期日记，每期最多 8 篇（按日期倒序）、单篇 1500 字截断；强制 `[YYYY-MM-DD]` 出处，日记里没有的必须明说；走云端模型。

泛化到本仓要回答的：

- 用本仓的 LLM 凭据链（同 `think`/`sense`）
- 注入预算（8 篇 × 1500 字）照搬还是标定——本仓 `docs` 审计有阈值标定的先例，照搬没有依据
- 跨域与跨仓——一天可能横跨 `context` 与 `journal` 两仓、多个域，注入顺序与出处粒度
- 脱敏前置（见教训四）

悬着：做不做、出处验收怎么算过。

### talk — 跨期对话

探路版：两个时期的「我」互相对话，默认 5 轮，主持人先发一次言；persona 只记得自己时期的事，回复限 150 字；transcript 落盘。

泛化到本仓要回答的：它是 `ask` 的多轮变体还是独立动词；transcript 是生成物，落 `QTFOUNDER_DATA`。

悬着：与 `ask` 的关系——同一实现两个入口，还是只做 `ask`。

### defer 与 digest — 挂起与消化

探路版：`defer` 零成本零延迟，只往 JSONL 队列追加一行，同日期幂等；`digest` 白天读队列，取当天日记加卡片关键词检索相似历史（最多 3 篇），生成四段消化稿（事实、情绪、类似事件、白天一个低成本动作），落盘出队；单天失败保留队列重试，索引缺失降级不阻断。

泛化到本仓要回答的：

- 归属——本仓已有 `execute collect`/`draw`（收待办、抽待办）与 think-reflect（思维记录），`defer` 的队列与谁重叠；这算日志工作流还是执行工作流
- 消化稿落哪——生成物走 `QTFOUNDER_DATA`，还是进 `data/` 成为资产（那是另一件事，要人拍）
- 队列的持久化与失败重试语义，本仓有没有对应物

悬着：归属与做不做——这组动词与本仓现有面重叠最多。

## 不做（已定的小项）

- 不做 `journal today`：写日报是 AGENTS 路径约定，不值一个子命令
- 不做自动提交、自动推送：移动完打印分层提交清单，提交走 git-commit 流程
- `list` 与 `archive` 不读正文：只认文件名与路径——这是这两个动词的定义，不是对其他动词的限制

## 验证（已定的两个动词）

- `journal list --period 2026-11` 与 `find data/context data/journal -name "20??-11-??.md"` 对得上，每条带出处
- `journal archive --period 2026-10` 预览列出的正是域根散页；`-y` 执行后 `qtfounder docs audit` 的 journal 层仍绿；再跑一遍零移动
- 全程无网络调用，断网可跑
