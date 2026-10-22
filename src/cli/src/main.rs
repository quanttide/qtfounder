//! qtfounder-cli：量潮创始人 CLI。
//!
//! 添加新功能三步：
//! 1. 在 `src/` 下新建模块（如 `feature.rs`），用 `mod` 声明
//! 2. 在 `CliCommand` 枚举里注册一个子命令分支
//! 3. 把该功能的依赖加回 `Cargo.toml`

mod code;
mod health;
mod language;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "qtfounder", version, about = "量潮创始人 CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<CliCommand>,
}

/// 子命令注册表：新功能在这里加一个枚举分支。
#[derive(Subcommand)]
enum CliCommand {
    /// 代码语用检查：词汇、句法、语篇、语用四层
    Code(code::CodeArgs),
    /// 语言检查：落点、术语、数字、句法、表格、语用七层
    Language(language::LanguageArgs),
    /// 健康检查：从会话库看消耗来自哪里、落在助手哪个环节
    Health(health::HealthArgs),
}

fn main() {
    match Cli::parse().command {
        Some(CliCommand::Code(args)) => match code::run_code(&args) {
            Ok(exit_code) => std::process::exit(exit_code),
            Err(message) => {
                eprintln!("{message}");
                std::process::exit(2);
            }
        },
        Some(CliCommand::Language(args)) => match language::run_language(&args) {
            Ok(exit_code) => std::process::exit(exit_code),
            Err(message) => {
                eprintln!("{message}");
                std::process::exit(2);
            }
        },
        Some(CliCommand::Health(args)) => match health::run_health(&args) {
            Ok(exit_code) => std::process::exit(exit_code),
            Err(message) => {
                eprintln!("{message}");
                std::process::exit(2);
            }
        },
        None => {
            println!("qtfounder：暂无子命令。");
            println!("新功能在 src/ 下新增模块，并在 CliCommand 注册。");
        }
    }
}
