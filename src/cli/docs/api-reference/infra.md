# `infra` 接口

## 一、命令面

```
qtfounder infra add <NAME> --on <EVENT> --run <CMD>
qtfounder infra add <NAME> --every <SPEC> --run <CMD>
qtfounder infra list [--json]
qtfounder infra remove <NAME>
qtfounder infra run <NAME>
qtfounder infra fire <EVENT>
qtfounder infra emit <NAME>
```

`--on` 与 `--every` 互斥，必须给一个。`--run` 的命令经 `sh -c` 执行。

## 二、`add`

**NAME**：表内唯一，重名以退出码 2 报错。要改一条先 `remove` 再 `add`。

你给的事件名与周期串都原样存，CLI 两样都不解释、不校验——事件名不问是不是合法的 git 钩子名，周期串不翻译成结构化时间。为什么这么放，见[本地与云端](../dev-guide/local-and-cloud.md)。

## 三、`list`

缺省打人读的表，`--json` 给机器读：

```json
{
  "path": "/home/u/.local/share/qtfounder/infra/subscriptions.json",
  "subscriptions": [
    {
      "name": "gate",
      "trigger": { "kind": "event", "name": "pre-commit" },
      "run": "qtfounder git hook run pre-commit",
      "enabled": true
    }
  ]
}
```

`trigger.kind` 取 `event` 与 `every` 两个值。`every` 那种的 `name` 换成 `spec`，值是驱动方能读的周期串。

## 四、`run` 与 `fire`

**`run <NAME>`**：跑一条，返回被跑命令的退出码。已停用的打一句提示退 0，不跑。

**`fire <EVENT>`**：跑所有盯这个事件且已启用的订阅，按表内顺序。第一条失败的退出码就是它的退出码，全过退 0。

**没有订阅者不是错**，打一句「事件「x」没有订阅者」退 0——CLI 不解释事件名，分不清拼错和「合法但没人订」。

## 五、`emit`

按触发方式打印驱动方那一行，不写任何东西：

- `event` → git 钩子片段 `exec qtfounder infra fire <EVENT>`；
- `every` → crontab 行 `<SPEC> qtfounder infra run <NAME>`。

## 六、退出码

退出码 `0` 是跑通，`fire` 全过或这个事件没人听都算。`1` 表示 `fire` 有订阅者跑失败。`2` 表示用法错：名字为空、命令为空、`--on` 与 `--every` 同给、或重名。`3` 表示目标缺席：`remove`、`run`、`emit` 点了一条表里没有的。

`run` 不套这四码，它原样返回被跑命令的退出码。

## 七、数据放哪

表文件是 `<数据根>/infra/subscriptions.json`。数据根按三级解析：先看 `QTFOUNDER_DATA` 这个环境变量；没有再看 XDG 的 `$XDG_DATA_HOME`，拼上应用目录名；再没有才落到家目录下的 `~/.local/share/qtfounder`。`data`、`execute` 用的是同一套，所以三处的文件在同一个地方。

第一次 `add` 会自己建目录；文件不在就是空表，不算错。想停用一条订阅，把它写成 `enabled: false`——记录还留在表里，只是不再被派发。
