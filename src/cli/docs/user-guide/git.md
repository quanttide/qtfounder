# `qtfounder git` 用户指南

`git` 这一组管工作区的 Git 配置，目前只有钩子。它只改 `core.hooksPath`，不改钩子里的内容，也不替你写脚本。

钩子目录固定在仓根的 `.githooks`，主仓与各层子模块共用同一份。

## 一、`install`：把配置指过去

```bash
qtfounder git hook install
qtfounder git hook install --root <仓>
qtfounder git hook install --force
```

不给 `--root` 就用当前目录所在的仓。命令先把根仓与所有子模块列出来，再逐个把 `core.hooksPath` 指到根仓的 `.githooks`。

某个仓已经指向别处时不覆盖，跳过并退 `1`；要接管就加 `--force`。未初始化的子模块也跳过，这类仓要先 `git submodule update --init`。

装之前会检查 `.githooks` 里的脚本。除 `.log` 以外的文件都当脚本看待，没有执行位就报错退出 `2`，一个配置也不改。

## 二、`status`：查有没有漂移

```bash
qtfounder git hook status
qtfounder git hook status --json
```

每个仓报一种状态：一致、未安装、指向别处、钩子目录不存在、脚本不可执行、子模块未初始化。

全部都一致退 `0`，有一个不是就退 `1`。适合接在提交前或发布前检查。

`--json` 输出 `repos` 数组，每项有 `path`、`state` 与可选的 `detail`，顶层 `ok` 表示是否全部一致。

## 三、`uninstall`：只卸自己装的

```bash
qtfounder git hook uninstall
```

只取消指向本仓 `.githooks` 的配置。指向别处的保留原样，输出里单列出来，避免误删别人的全局钩子。

## 四、绝对路径与搬家

写进配置的是绝对路径。仓库搬家或换机器后，旧路径可能仍然存在，`status` 会把它报成「指向别处」，不会误判为一致。

搬完先跑 `status`，再跑一次 `install --force` 即可。子模块单独克隆时没有根仓的 `.githooks`，也需要回到工作区后重跑安装。
