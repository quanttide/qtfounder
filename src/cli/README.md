# qtfounder-cli

量潮创始人 CLI 工具。七个子命令：`code` 查源码，`language` 查文字，`docs` 查文档结构，`think` 管认知（含负载诊断与反思记录），`meta` 判归位，`skill` 装 Skill，`audit` 出体检报告。

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
| `qtfounder docs` | 文档审计：表格、列表、密度、加粗、分割线五层，判这段内容该不该是这个形状；`docs check` 比对技能索引与目录。用法见 [docs/user-guide/docs.md](docs/user-guide/docs.md) |
| `qtfounder think` | 认知：`load` 从会话库看消耗落在哪，`reflect` 按 CBT 那套记一条并到点回填，`model` 看当前模型。用法见 [docs/user-guide/think.md](docs/user-guide/think.md) |
| `qtfounder meta` | 归位：`classify` 给一段内容一张落位卡——域、格、档、写法；判不了只问一句，套不上就不收。用法见 [docs/user-guide/meta.md](docs/user-guide/meta.md) |
| `qtfounder skill` | Skill：构建时把主仓 `.agents/skills` 嵌进二进制，`list` 看目标仓装到哪一步，`install` 装出去，`check` 只查差异。用法见 [docs/user-guide/skill.md](docs/user-guide/skill.md) |
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
mkdir -p ~/.config/qtfounder/code ~/.config/qtfounder/language ~/.config/qtfounder/think ~/.config/qtfounder/docs ~/.config/qtfounder/meta
cp src/code/default-contract.toml     ~/.config/qtfounder/code/
cp src/language/default-contract.toml ~/.config/qtfounder/language/
cp src/language/sense-prompt.md       ~/.config/qtfounder/language/
cp src/think/load/prompt.md           ~/.config/qtfounder/think/
cp src/think/default-contract.toml    ~/.config/qtfounder/think/model.toml
cp src/docs/default-contract.toml     ~/.config/qtfounder/docs/
cp src/meta/default-contract.toml     ~/.config/qtfounder/meta/
```

契约查找顺序：`--rules` ／ `--model` → 环境变量（`QTFOUNDER_CODE_CONTRACT`／`QTFOUNDER_LANGUAGE_CONTRACT`／`QTFOUNDER_DOCS_CONTRACT`／`QTFOUNDER_META_CONTRACT`／`QTFOUNDER_THINK_MODEL`）→ 从当前目录逐级往上找 → `~/.config/qtfounder/<模块>/`。四道都找不到就以退出码 2 报错，**不内置兜底**——静默回退会让人以为阈值生效了。

`think` 是例外：认知模型有内置兜底。它是记录本，不该因为找不到配置文件就记不了账；带 `--model` 或把模型装到用户级目录，就能覆盖内置那份。

最后一条容易漏：只装二进制不装契约，在任何非源码目录里跑都会直接报错。
