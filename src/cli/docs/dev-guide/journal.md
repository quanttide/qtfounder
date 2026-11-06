# journal：列出与归档

状态：设计稿，未实现。

范围是本仓日志的两件事：把散在三处的日记**列出来**，把该归位的**归位**。写日报本身是 AGENTS 已写明的路径约定，不新增命令；问答、索引、挂起消化都不在本设计里。

研究对象是 thera diary（独立项目的日记命令组，与本仓无任何依赖）：只借它的经验与教训，不调用它、不共享配置与文件格式，两边各管各的正文。

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
4. **生成物落 XDG、正文只读**：本仓已有 `QTFOUNDER_DATA` → `$XDG_DATA_HOME/qtfounder`（`src/core/store.rs`）。将来 `list`/`archive` 长出任何清单与缓存，都落这里，不进 git。
5. **批量单点失败不中断**：它的索引按篇容错、消化按天保留队列重试——`archive` 照此：单页移动失败（权限、目标占用）跳过继续，最后汇总失败清单，退出码非 0。
6. **进度强制刷 stdout**：它修过管道下块缓冲导致进度不可见（`[n/N]` 强制 flush）。批量归档的进度行照此写。
7. **动用户数据先确认**：它的仓内 AGENTS 写死「涉及用户数据（移动、删除、重命名）前必须先确认」。`archive` 默认只预览。
8. **测试不碰真实环境**：它修过测试往真实 cwd 写、测试发真网络。本仓用临时目录加注入桩的惯例已有，新模块照办。
9. **局限写在文档里**：它把「检索不认同义词」直说。本设计的局限（只按文件名与路径解析、不读正文）同样写明。

## 避免的教训

1. **生成物写进仓库**：它 2026-11-06 才把硬编码 `data/` 的导出、往真实 cwd 写的测试、默认落仓的对话记录统一改成 XDG（提交 `dbcb534`），是补丁不是设计。本设计第一天就不产生仓内文件。
2. **根目录碎片转嫁给用户**：它的日记根要 `--journal` 手工指，文档还得教「新日志在哪个根」。本仓碎片更甚（三处），`list` 与 `archive` 必须原生解析三处，不加参数、不设环境变量。
3. **命令面铺开再砍**：它 0.2.0 一次删掉 auto-commit、doc-check、submodule-sync、workflow 四个命令。本设计只上 `list` 与 `archive`，多一个都先等需求。
4. **日记原文进 LLM 前不脱敏**：它是出过内容才补的脱敏提交（`ae9e51c`，抽取 prompt 的示例与测试数据里带过真实日记内容）。本设计无 LLM 面；将来若加，脱敏是前置条件，不是补丁。
5. **归档顺手改写**：迁移纪律是只移动不改写。`archive` 不改文件名、不改正文、不改域；发现结构不对（某页该不该属这个域）就停下来报，不替人改。

## 命令面

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

## 不做

- 不做问答、索引、挂起消化：本仓没有这条需求面，也不为它引 LLM
- 不做 `journal today`：写日报就是按 AGENTS 路径建文件，不值一个子命令
- 不做自动提交、自动推送
- 不读正文：`list` 与 `archive` 只认文件名与路径

## 验证

- `journal list --period 2026-11` 与 `find data/context data/journal -name "20??-11-??.md"` 对得上，每条带出处
- `journal archive --period 2026-10` 预览列出的正是域根散页；`-y` 执行后 `qtfounder docs audit` 的 journal 层仍绿；再跑一遍零移动
- 全程无网络调用，断网可跑
