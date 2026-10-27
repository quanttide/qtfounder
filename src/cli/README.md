# qtfounder-cli

量潮创始人 CLI 工具。十一个子命令：`code` 查源码，`language` 查文字，`docs` 查文档结构，`write` 查表达，`think` 管认知（含负载诊断与反思记录），`meta` 判归位与看归位边，`asset` 看资产契约、清单、关系与维护规则，`product` 记需求差，`agent` 装 Skill、查写法与看命令关系，`git` 管钩子配置，`audit` 出体检报告。

## 开发

```bash
cargo run              # 无子命令时输出接入提示
cargo build --release
cargo test
```

## 子命令

| 命令 | 说明 |
|---|---|
| `qtfounder code` | 代码语用检查：词汇、句法、语篇、语用四层，判下一个维护者能不能看懂。用法见 [docs/user-guide/code.md](docs/user-guide/code.md) |
| `qtfounder language` | 语言检查：落点、自造术语、体系词、数字、句法、语用六层，判这段文字是不是在跟读者说话 |
| `qtfounder docs` | 文档审计：表格、列表、密度、加粗、分割线、front matter 与日志模板七层，判这段内容该不该是这个形状；`docs check` 比对技能索引与目录。用法见 [docs/user-guide/docs.md](docs/user-guide/docs.md) |
| `qtfounder write` | 表达审计：体裁、读者、主题三轴，加空话、口号、指路、结构四层。用法见 [docs/user-guide/write.md](docs/user-guide/write.md) |
| `qtfounder think` | 认知：`load` 从会话库看消耗落在哪，`reflect` 按 CBT 那套记一条并到点回填，`model` 看当前模型。用法见 [docs/user-guide/think.md](docs/user-guide/think.md) |
| `qtfounder meta` | 归位四部分：`model` 看两族九宫格，`relation` 看迁移与分辨句，`axiom` 看前提，`classify` 给落位卡。用法见 [docs/user-guide/meta.md](docs/user-guide/meta.md) |
| `qtfounder asset` | 资产：`contract` 看每类资产的边界、分档与写法，`catalog` 看二十种资产在不在，`relation` 看角色与迁入边，`maintain` 整理 context 并搬进长期记忆仓。用法见 [docs/user-guide/asset.md](docs/user-guide/asset.md) |
| `qtfounder agent` | Agent：`skill` 一组——`list`／`check` 看装到哪一步，`install` 装到系统位置 `~/.agents/skills`，`lint` 查名字、说明与正文合不合标准；`relation` 看 Skill 引用了哪些命令、哪些命令组还没 Skill。用法见 [docs/user-guide/agent.md](docs/user-guide/agent.md) |
| `qtfounder git` | Git：`hook` 一组——`install`／`status`／`uninstall` 管主仓与子模块的 `core.hooksPath`，`run` 执行钩子；`submodule check` 查指针与推送状态。用法见 [docs/user-guide/git.md](docs/user-guide/git.md) |
| `qtfounder audit` | 文字体检：合并 `language` 与 `docs`，按「一个词、一条规则」归堆，每期只报一处。用法见 [docs/user-guide/audit.md](docs/user-guide/audit.md) |

## 添加新功能

1. `src/` 下新建模块（如 `feature.rs`），在 `main.rs` 用 `mod` 声明
2. `CliCommand` 枚举注册子命令分支
3. 功能依赖加回 `Cargo.toml`

## 许可

Apache 2.0

## 装到本地

```bash
cargo build --release
install -m755 target/release/qtfounder ~/.local/bin/qtfounder

# 契约装到用户级目录，这样脱离源码树也能跑、改阈值不用重编译
mkdir -p ~/.config/qtfounder/code ~/.config/qtfounder/language ~/.config/qtfounder/think ~/.config/qtfounder/docs ~/.config/qtfounder/write ~/.config/qtfounder/meta
cp src/code/default-contract.toml     ~/.config/qtfounder/code/
cp src/language/default-contract.toml ~/.config/qtfounder/language/
cp src/language/sense-prompt.md       ~/.config/qtfounder/language/
cp src/think/load/prompt.md           ~/.config/qtfounder/think/
cp src/think/default-contract.toml    ~/.config/qtfounder/think/model.toml
cp src/docs/default-contract.toml     ~/.config/qtfounder/docs/
cp src/write/default-contract.toml    ~/.config/qtfounder/write/
cp src/meta/default-contract.toml     ~/.config/qtfounder/meta/
```

契约查找顺序：`--rules` ／ `--model` → 环境变量（`QTFOUNDER_CODE_CONTRACT`／`QTFOUNDER_LANGUAGE_CONTRACT`／`QTFOUNDER_DOCS_CONTRACT`／`QTFOUNDER_WRITE_CONTRACT`／`QTFOUNDER_META_CONTRACT`／`QTFOUNDER_THINK_MODEL`）→ 从当前目录逐级往上找 → `~/.config/qtfounder/<模块>/`。四道都找不到就以退出码 2 报错，**不内置兜底**——静默回退会让人以为阈值生效了。

`think` 是例外：认知模型有内置兜底。它是记录本，不该因为找不到配置文件就记不了账；带 `--model` 或把模型装到用户级目录，就能覆盖内置那份。

最后一条容易漏：只装二进制不装契约，在任何非源码目录里跑都会直接报错。
