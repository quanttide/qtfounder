# qtfounder-cli

量潮创始人 CLI 工具。十五个子命令分两组：查与审的是 `code` 查源码、`language` 查文字、`docs` 查文档结构、`write` 查表达、`audit` 出体检报告、`design` 走查界面交付物的字号与主按钮。

管创始人日常的是 `think` 管认知（含负载诊断与反思记录）、`meta` 判归位与看归位边、`asset` 看资产契约、清单、关系与维护规则、`execute` 收待办并抽一件、`data` 管标注规则、任务包与结果、`product` 记需求差、`infra` 管订阅登记与派发、`agent` 装 Skill、查写法与看命令关系、`git` 管钩子配置。

## 开发

```bash
cargo run              # 无子命令时输出接入提示
cargo build --release
cargo test
```

## 子命令

| 命令 | 说明 |
|---|---|
| `qtfounder code` | 代码语用检查：词汇、句法、语篇、语用四层，判下一个维护者能不能看懂；`code reflect` 给任务后的架构反思清单，`code review` 给意图审查骨架与锚点。用法见 [docs/user-guide/code.md](docs/user-guide/code.md) |
| `qtfounder language` | 语言检查：`placement` 归到哪、`banned` 自造术语、`required` 体系词、`numbers` 数字、`syntax` 句法、`voice` 语用六层，判这段文字是不是在跟读者说话 |
| `qtfounder docs` | 文档审计：表格、列表、密度、加粗、分割线、front matter 与日志模板七层，判这段内容该不该是这个形状；`docs format` 先数格式元素（标题、表格、列表、加粗、分割线、块引用、代码块）再审计；`docs check` 比对技能索引与目录。用法见 [docs/user-guide/docs.md](docs/user-guide/docs.md) |
| `qtfounder write` | 表达审计：体裁、读者、主题三轴，加空话、口号、指路、结构四层。用法见 [docs/user-guide/write.md](docs/user-guide/write.md) |
| `qtfounder think` | 认知：`load` 从会话库看消耗落在哪，`reflect` 按 CBT 那套记一条并到点回填，`model` 看当前模型。用法见 [docs/user-guide/think.md](docs/user-guide/think.md) |
| `qtfounder meta` | 归位四部分：`model` 看两族九宫格，`relation` 看迁移与分辨句，`axiom` 看前提，`classify` 给落位卡。用法见 [docs/user-guide/meta.md](docs/user-guide/meta.md) |
| `qtfounder asset` | 资产：`contract` 看每类资产的边界、分档与写法，`catalog` 看二十种资产在不在，`relation` 看角色与迁入边，`axiom` 看离不开物理形态的前提，`maintain` 整理 context 并搬进长期记忆仓。用法见 [docs/user-guide/asset.md](docs/user-guide/asset.md) |
| `qtfounder agent` | Agent：`skill` 一组——`list`／`check` 看装到哪一步，`install` 装到系统位置 `~/.agents/skills`，`lint` 查名字、说明与正文合不合标准；`relation` 看 Skill 引用了哪些命令、哪些命令组还没 Skill。用法见 [docs/user-guide/agent.md](docs/user-guide/agent.md) |
| `qtfounder git` | Git：`hook` 一组——`install`／`status`／`uninstall` 管主仓与子模块的 `core.hooksPath`，`run` 执行钩子；`submodule check` 查指针与推送状态。用法见 [docs/user-guide/git.md](docs/user-guide/git.md) |
| `qtfounder audit` | 文字体检：合并 `language` 与 `docs`，按「一个词、一条规则」归堆，每期只说一处。用法见 [docs/user-guide/audit.md](docs/user-guide/audit.md) |
| `qtfounder execute` | 待办的收与抽：`collect` 收待办进池子（同文本去重、`--stdin` 批量、优先级缺省 1），`draw` 按优先级加权抽一件。用法见 [docs/user-guide/execute.md](docs/user-guide/execute.md) |
| `qtfounder data` | 标注三样：`rule` 设计与存取规则（机器名/显示名/定义/正反例），`task` 看 studio 导出的任务包，`result` 导入标注端 JSONL 并按标签统计；XDG 存储，与 studio、lite 共用。用法见 [docs/user-guide/data.md](docs/user-guide/data.md) |
| `qtfounder product` | 产品：`clarify` 记一次澄清、`list` 看需求差、`organize` 读聚类与归档、`clear` 清缓存。用法见 [docs/user-guide/product.md](docs/user-guide/product.md) |
| `qtfounder infra` | 订阅登记与派发：`add` 记一条周期或事件订阅，`list`／`remove` 管表，`run`／`fire` 执行，`emit` 打驱动方那一行。用法见 [docs/user-guide/infra.md](docs/user-guide/infra.md) |
| `qtfounder design` | 交互设计走查：数 HTML 交付物的字号（第九条中文微排版）与主按钮（第五条主行动唯一），`design principles` 摊开十二条原则。用法见 [docs/user-guide/design.md](docs/user-guide/design.md) |

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
