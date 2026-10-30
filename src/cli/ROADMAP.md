# CLI 路线图：infra 走向云端

给接手这段代码的会话：方向已定案，验证走了一半，剩下的活按顺序列在下面。方向正本在 quanttide-founder 仓 `data/roadmap/qtfounder/infra.md`，设计立场在 [dev-guide/local-and-cloud](docs/dev-guide/local-and-cloud.md)，本文只记实现任务与验收。

## 目标

一个本地环境里的组件，可以映射到云端的一个集群：定时器本地是 crontab、云端是定时触发器；消息队列本地可部署、也可用云端。CLI 声明「什么触发、跑什么」，触发的执行机制属于环境。infra 从第一天就不自己实现调度，所以换环境时换的只是驱动方，表与命令面不动。

## 已定案，不要「修好」

以下三条是设计立场，不是没写完。改动前先读 dev-guide 那篇的第四节：

- 事件名不枚举、不校验。`pre-commit` 不校验是不是合法 git hook 名，一校验，云端总线的 `release.published` 就成了非法事件名
- 周期串不解析。`0 3 * * *` 不翻译成「每天三点」，一翻译，换成 systemd timer 时语义就变了
- 无人听的事件退 0 不报错。CLI 分不清拼错和「合法但没人订」，报假错比不报更坏

另有两件刻意不做：`emit` 只打印驱动方那一行，不替人装 crontab、不动仓里已有的 `.githooks`。

## 已验证（2026-10-30）

- 本仓 `.githooks/pre-commit` 改走 `exec qtfounder infra fire pre-commit`，钩子先 `cd` 主仓根——订阅命令不带 `--root`，靠 cwd 定位
- 正向：造一笔带禁用词的提交，经 fire 触发订阅表里的 gate，拦下退 1
- 反向：`infra remove gate` 后同一笔放行，fire 打「没有订阅者」退 0
- 子模块提交走的也是这条链路（hooks 装在各子模块）

订阅登记在本机 XDG 数据根 `~/.local/share/qtfounder/infra/subscriptions.json`，不在仓里。换机器或重新克隆后要重新 `infra add gate --on pre-commit --run "qtfounder git hook run pre-commit"`，否则钩子静默放行——这是 fire 的立场，不是故障。

## 任务一：补测试

`src/infra/` 目前零覆盖。照 `tests/` 现有文件的风格（真样本、反样本、临时目录隔离）补三处：

- store：add → load 字段不变的往返；重名 add 报错
- fire：无人听退 0；多条订阅时返回第一条失败的退出码
- run：返回被跑命令的退出码；停用的订阅不跑、退 0

验收：`cargo test` 全绿。

## 任务二：外部驱动方验证（不写代码）

换一个非 git 的驱动方消费同一张表，验证命令面不长新东西。这是云端可迁移在本地能验的那一半：

- 登记一条 `--every` 订阅，把 `emit` 打出的 crontab 行真装进 crontab，次日确认跑了、退出码对
- 登记一条非 git 事件名的订阅（如 `release.published`），从任意目录 `fire` 它

验收：两条都跑通，过程记回 `data/roadmap/qtfounder/infra.md` 的云端节。若发现命令面需要加东西，先停下——那说明形状要动，回到方向层再定。

## 任务三：平移到 qtcloud-infra

家的规矩（qtcloud-devops 定的先例）：这类工具的家只能在 qtcloud-* 独立仓，不在产品仓内部。它的接口稳定性不是产品仓给的，是外部消费者给的——根仓 AGENTS.md、各仓的 devops-release skill、发布流程都直接调 `qtcloud-devops release publish`；反过来「先在产品仓开发再分流」会让工具继承产品仓的发版节奏，拆出去时所有消费者的安装路径与文档还得再迁一次。所以工具独立演进、独立发版，要用就依赖它——`src/cli/Cargo.toml` 里写的是 `qtcloud-devops-cli = "0.8"` 依赖 crate，不是 vendor。

infra 同理：平移目标是 qtcloud-infra（量潮基础设施云，仓已存在），不是 qtcloud 主站；先在 qtcloud-infra 落地，再往 qtcloud 集成。前提：任务一、二都过。平移本身是把 `src/infra/`（mod.rs + store.rs）搬进 qtcloud-infra 的 CLI，注册同样的命令面，表各自独立（XDG 数据根按 CLI 名分家，订阅不共享——表的归属已定，不是开放问题）。

qtcloud-infra 侧真正要接的是第一个云端事件源：总线推事件 → `infra fire <事件名>` 跑通一次。那一段等有事件源再做，不急。

验收：云端事件经 fire 触发一条真实订阅，与本地走同一条命令面。

## 唯一可能动形状的地方

fire 不带载荷：只有事件名，订阅的 run 是固定命令串。本地世界没有东西能先验证它——crontab 行不带数据，git 钩子只说「发生了」；云端的队列消息必然带消息体。哪天某条订阅需要「按事件内容决定跑什么」（qtcloud 的 `release.published` 带版本号、`deploy.finished` 带环境名），fire 的签名要加东西（`--data` 或环境变量注入）。真到那天，先在 qtfounder 侧把载荷语义定了再平移，避免两边各长一版。
