# `qtfounder infra` 用户指南

`infra` 记下「什么事件、跑什么命令」，事件来了把命令交出去。它自己不调度——CLI 被调用就退出，没有进程活着等闹钟。
本机要一条存得住的事件总线，即事件排队等着订阅端、机器重启也不丢，见[事件总线](event-bus.md)。

## 一、先认清它不做什么

以下都是刻意的：

- 不写你的 crontab；
- 不动仓里已有的 `.githooks`；
- 不解释事件名与周期串。

不写 crontab、不动 `.githooks` 是往外系统里伸手，要单独一次决定。不解释事件名与周期串是设计立场，理由见开发指南[本地与云端](../dev-guide/local-and-cloud.md)。

## 二、登记

```bash
qtfounder infra add gate --on pre-commit --run "qtfounder git hook run pre-commit"
qtfounder infra add nightly --every "0 3 * * *" --run "qtfounder language docs --top 5"
```

名字在表内唯一，重名报错；要改先 `remove` 再 `add`。表落 `<数据根>/infra/subscriptions.json`，与 `data`、`execute` 同一个 XDG 约定。

## 三、接线：emit 只打印

```bash
$ qtfounder infra emit nightly
# crontab 行：`crontab -e` 加进去（不自动写你的 crontab）
0 3 * * * qtfounder infra run nightly
```

`--on` 的打 git 钩子片段，`--every` 的打 crontab 行。你把那一行贴过去，它不替你贴。

**接完之后本地和云端走的是同一条入口。** `.githooks/pre-commit` 里那行 `exec qtfounder infra fire pre-commit`，与将来云端总线推 `pre-commit` 时调的，是同一个命令、同一个入口。

## 四、派发：fire 与 run

```bash
qtfounder infra fire pre-commit   # 事件来了：跑所有盯它且已启用的订阅
qtfounder infra run nightly       # 跑指定一条，返回它的退出码
```

`fire` 一个没人听的事件**不报错**，退 0：

```text
事件「post-release」没有订阅者
```

它分不清拼错和「合法但没人订」——不解释事件名，就无从校验。报一个假错比不报错更坏。

## 五、停用不是删除

`enabled: false` 之后 `fire` 与 `run` 都跳过它，记录还在。「暂时不想跑」与「永久去掉」一旦合起来，过两个月就分不清当初是关了还是删了。

## 六、退出码

`run` 返回被跑命令的退出码；`fire` 返回第一条失败的退出码，全过退 0。目标缺席退 3，用法错退 2。
