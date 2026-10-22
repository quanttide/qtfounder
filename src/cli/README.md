# qtfounder-cli

量潮创始人 CLI 工具。当前为脚手架：入口与结构保留，功能待添加。

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
| `qtfounder language` | 语言检查：落点、自造术语、体系词、数字、句法、表格、语用七层，判这段文字是不是在跟读者说话 |

## 添加新功能

1. `src/` 下新建模块（如 `feature.rs`），在 `main.rs` 用 `mod` 声明
2. `CliCommand` 枚举注册子命令分支
3. 功能依赖加回 `Cargo.toml`

## 许可

Apache 2.0
