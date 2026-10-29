# TODO

`execute` 命令与 `execute-collect` Skill 的代码、测试已写完，收尾未做，故暂存于此。全部做完后删除本文件。

## 一、验证

- [ ] 全量 `cargo test`（上次被中断，最后一次全绿只到单测 6/6 与 `--test execute` 10/10；`agent_skill`、`registry` 的九条同步改后未验证）

## 二、文档同步（新增命令必须配文档，照 `design` 的先例）

- [ ] 新建 `docs/user-guide/execute.md`：待办的收与抽；命令面 `qtfounder execute collect [<text>...] [--priority <n>] [--stdin] [--json]` 与 `qtfounder execute draw [--json]`；退出码 0 成功（含去重）/ 2 用法错 / 3 池子空；`priority` 缺省 1 是抽签权重，0 永远抽不中故拒收
- [ ] 新建 `docs/api-reference/execute.md`：照 `product.md` 体例——命令面、collect、draw、错误表（JSON 信封）、数据放哪（`execute.json`，复用 `product` 的数据目录与 id 方案）
- [ ] `docs/myst.yml`：toc 加 `execute.md` 两行；description「十二个子命令」→十三个
- [ ] `docs/index.md`：开头「十二个子命令」句加 `execute`；用户指南、接口参考各加一行
- [ ] `docs/user-guide/agent.md` + `docs/api-reference/agent.md`：「八条」→「九条」，名单加 `execute-collect`

## 三、主仓数量词（Skill 数量词全局同步）

- [ ] 主仓 `AGENTS.md`、`CONTRIBUTING.md`、`.agents/README.md`：「八条随 qtfounder 发布」→九条，名单加 `execute-collect`
- [ ] 主仓 `AGENTS.md` 快速索引加一行（如「抽一件待办去做 → `qtfounder execute draw`」）

## 四、CHANGELOG（cli 仓 Unreleased/Added）

- [ ] `execute-collect` Skill 随 CLI 发布（第九条）：从 AI 协作上下文抽取待办
- [ ] `qtfounder execute collect` / `execute draw`：收待办进池子（同文本去重、`--stdin` 批量、优先级缺省 1）；按优先级加权抽一件，抽走进 `drawn` 留痕

## 五、审计与构建

- [ ] 新增中文文档跑审计：`target/debug/qtfounder language <文件>` + `docs format <文件>`（在根仓跑，契约能向上找到），新增行零命中禁用词（清单见主仓 `data/roadmap/qtfounder/language.md`）
- [ ] 重新 `cargo build`（Skill 正文编译期嵌入）
- [ ] 可选：`qtfounder agent skill lint skills/` 验证新 skill 过七道检查

## 六、提交推送（顺序不能反）

- [ ] 先在 `apps/qtfounder` 子模块提交推送
- [ ] 回主仓 bump 指针提交推送；根仓只 `git add` 本次路径，不 `git add -A`
