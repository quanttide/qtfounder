# `git` 接口

## 一、命令面

```
qtfounder git hook install [--root PATH] [--force] [--json]
qtfounder git hook status  [--root PATH] [--json]
qtfounder git hook uninstall [--root PATH] [--json]
qtfounder git hook run <NAME> [--root PATH]
qtfounder git submodule check [--root PATH] [--pushed] [--json]
```

**`--root`** 不给时用当前目录所在的 Git 仓（`git rev-parse --show-toplevel`）。注意它给的是「当前这个仓」的根：`data/roadmap` 这类子模块返回它自己，不是外层仓。

## 二、`hook install`

把主仓与各层子模块的 `core.hooksPath` 指到 `<root>/.githooks`。钩子脚本本身不动——它只改配置。

**`--force`**：已指向别处时也覆盖。不给时，指向别处的仓会被跳过并在报告里点名。

仓里已有的 `.githooks/` 内容由仓自己维护，`install` 不写它。想让 `infra` 的订阅接进钩子，用 `qtfounder infra emit` 拿片段自己追加。

## 三、`hook status`

逐仓查 `core.hooksPath` 有没有漂移。JSON 形：

```json
{
  "root": "/…/quanttide-founder",
  "hooks": "/…/quanttide-founder/.githooks",
  "ok": false,
  "repos": [
    { "path": ".", "state": "ok", "detail": null },
    { "path": "apps/qtcloud", "state": "unset", "detail": null }
  ]
}
```

`state` 六种：`ok`、`unset`（没配）、`other`（指向别处，值在 `detail`）、`broken`、`not_executable`、`uninitialized`。有漂移退 1。

## 四、`hook run <NAME>`

由 `.githooks/` 里的脚本调用，人一般不直接跑。它按钩子名执行对应检查，返回检查的退出码。

## 五、`submodule check`

逐个子模块查三处：指针与父仓记录是否一致、有没有未提交改动、HEAD 有没有推到远端。JSON 形：

```json
{
  "root": "/…/quanttide-founder",
  "ok": false,
  "pushed": false,
  "submodules": [
    {
      "path": "apps/qtfounder",
      "recorded": "b96844a…",
      "head": "61a1278…",
      "issues": [ { "kind": "drift", "detail": null } ]
    }
  ]
}
```

`issues[].kind` 六种：`uninitialized`、`drift`（指针与记录对不上）、`dirty`（有未提交改动）、`unpushed`（HEAD 不在远端）、`pointer-unpushed`（记录的指针不在远端）、`pointer-missing`（指针在远端找不到）。有问题退 1。

**`--pushed`** 只查推送安全那两项——未推送的提交与指针不在远端，跳过指针一致性。发版前用它比全查快。

## 六、退出码

退出码 `0` 是全部正常。`1` 表示有漂移、有未提交改动、或有未推送的提交。`2` 表示用法错，或目标根本不在 Git 仓里。

`git` 这一组不用码 3——「不在仓里」按用法错处理。`hook run` 也不套这三码，它原样返回所执行钩子的退出码。
